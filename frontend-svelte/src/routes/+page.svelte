<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import Toaster from '#lib/components/ui/sonner/sonner.svelte';
	import { toast } from 'svelte-sonner';

	// Types and constants
	import {
		CAT_KEY,
		PROJECT_KEY,
		SORT_FIELD_KEY,
		SORT_DIR_KEY,
		THEME_KEY,
		EXPORT_DIR_KEY,
		type Task,
		type Category,
		type Project,
		type Tab,
		type Page,
		type SortField,
		type SortDir
	} from '#lib/types.js';

	// Views
	import ProjectsView from '#lib/components/views/ProjectsView.svelte';
	import CategoriesView from '#lib/components/views/CategoriesView.svelte';
	import TasksView from '#lib/components/views/TasksView.svelte';
	import AboutView from '#lib/components/views/AboutView.svelte';

	// Dialogs
	import LockOverlay from '#lib/components/dialogs/LockOverlay.svelte';
	import ExportDialog from '#lib/components/dialogs/ExportDialog.svelte';
	import ImportDialog from '#lib/components/dialogs/ImportDialog.svelte';
	import ProjectDialog from '#lib/components/dialogs/ProjectDialog.svelte';
	import ProjectPickerDialog from '#lib/components/dialogs/ProjectPickerDialog.svelte';
	import CategoryDialog from '#lib/components/dialogs/CategoryDialog.svelte';
	import MoveTaskDialog from '#lib/components/dialogs/MoveTaskDialog.svelte';
	import ConfirmDialog from '#lib/components/dialogs/ConfirmDialog.svelte';

	// Core State
	let tasks = $state<Task[]>([]);
	let cats = $state<Category[]>([]);
	let projects = $state<Project[]>([]);
	let activeProject = $state<number | null>(null);
	let page = $state<Page>('home');
	let prevPage = $state<Page>('home');
	let tab = $state<Tab>('active');
	let activeCat = $state<number | null>(null);
	let newCat = $state<number | null>(null);
	let sortField = $state<SortField>('cat_pos');
	let sortDir = $state<SortDir>('asc');
	let exportDirectory = $state('');

	// Dialog & Action State
	let locked = $state(true);
	let pinSet = $state(false);
	let projectPickerOpen = $state(false);
	let exportDialogOpen = $state(false);
	let importDialogOpen = $state(false);
	let projectDialogMode = $state<'add' | 'edit' | null>(null);
	let editingProject = $state<Project | null>(null);
	let categoryDialogMode = $state<'add' | 'rename' | null>(null);
	let editingCategory = $state<Category | null>(null);
	let moveTask = $state<Task | null>(null);
	let confirmState = $state<{
		msg: string;
		run: () => Promise<void>;
		variant?: 'destructive' | 'default';
		actionLabel?: string;
	} | null>(null);

	// Derived
	const activeProjectItem = $derived(projects.find((p) => p.id === activeProject));

	// Theme
	let dark = $state(typeof document !== 'undefined' && document.documentElement.classList.contains('dark'));
	function toggleTheme() {
		dark = !dark;
		document.documentElement.classList.toggle('dark', dark);
		localStorage.setItem(THEME_KEY, dark ? 'dark' : 'light');
	}

	function lockApp() {
		locked = true;
	}

	function openAbout() {
		prevPage = page;
		page = 'about';
	}

	function askConfirm(
		msg: string,
		run: () => Promise<void>,
		variant?: 'destructive' | 'default',
		actionLabel?: string
	) {
		confirmState = { msg, run, variant, actionLabel };
	}

	// Backend API Calls
	async function refresh() {
		if (activeProject === null) {
			tasks = [];
			return;
		}
		tasks = await invoke<Task[]>('list_tasks', { projectId: activeProject });
	}

	async function reloadCats() {
		if (activeProject === null) {
			cats = [];
			return;
		}
		cats = await invoke<Category[]>('list_categories', { projectId: activeProject });
	}

	async function reloadProjects() {
		projects = await invoke<Project[]>('list_projects');
	}

	async function pickProject(id: number) {
		if (!projects.some((p) => p.id === id)) return;
		activeProject = id;
		localStorage.setItem(PROJECT_KEY, String(id));
		page = 'tasks';
		projectPickerOpen = false;
		activeCat = null;
		localStorage.setItem(CAT_KEY, 'all');
		await reloadCats();
		newCat = cats[0]?.id ?? null;
		await refresh();
	}

	function openProjectDialog(mode: 'add' | 'edit', project?: Project) {
		editingProject = project ?? null;
		projectDialogMode = mode;
	}

	function deleteProject(selected: Project) {
		askConfirm(`Hapus space "${selected.name}"?`, async () => {
			try {
				await invoke('delete_project', { id: selected.id });
				await reloadProjects();
				if (activeProject === selected.id) {
					activeProject = null;
					tasks = [];
					cats = [];
					localStorage.removeItem(PROJECT_KEY);
					page = 'home';
				}
				toast.success(`Space "${selected.name}" berhasil dihapus`);
			} catch (e) {
				toast.error(String(e));
			}
		});
	}

	async function setProjectStatus(id: number, status: 'active' | 'completed' | 'archived') {
		try {
			await invoke('set_project_status', { id, status });
			await reloadProjects();
			const statusLabel =
				status === 'completed'
					? 'ditandai selesai'
					: status === 'archived'
						? 'diarsipkan'
						: 'diaktifkan kembali';
			toast.success(`Space berhasil ${statusLabel}`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	function openCategoryDialog(mode: 'add' | 'rename', cat?: Category) {
		editingCategory = cat ?? (mode === 'rename' ? (cats.find(c => c.id === activeCat) ?? cats[0] ?? null) : null);
		categoryDialogMode = mode;
	}

	function deleteCategory(target?: Category) {
		const cur = target ?? cats.find(c => c.id === activeCat);
		if (!cur) return;
		askConfirm(`Hapus kategori "${cur.name}"?`, async () => {
			try {
				await invoke('delete_category', { id: cur.id, projectId: activeProject });
				await reloadCats();
				if (!cats.some((c) => c.id === activeCat)) {
					activeCat = cats.length ? cats[0].id : null;
					localStorage.setItem(CAT_KEY, activeCat !== null ? String(activeCat) : 'all');
				}
				if (newCat === null || !cats.some((c) => c.id === newCat)) newCat = cats[0]?.id ?? null;
				await refresh();
				toast.success(`Kategori "${cur.name}" berhasil dihapus`);
			} catch (e) {
				toast.error(String(e));
			}
		});
	}

	async function moveCategory(index: number, direction: -1 | 1) {
		const targetIndex = index + direction;
		if (targetIndex < 0 || targetIndex >= cats.length || activeProject === null) return;
		const reordered = [...cats];
		const [moved] = reordered.splice(index, 1);
		reordered.splice(targetIndex, 0, moved);
		cats = reordered;
		try {
			await invoke('reorder_categories', {
				ids: reordered.map((c) => c.id),
				projectId: activeProject
			});
			await reloadCats();
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function handleReorderProjects(ids: number[]) {
		try {
			await invoke('reorder_projects', { ids });
			await reloadProjects();
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function handleReorderCategories(ids: number[]) {
		if (activeProject === null) return;
		try {
			await invoke('reorder_categories', {
				ids,
				projectId: activeProject
			});
			await reloadCats();
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function setCatColorQuick(cat: Category, color: string) {
		if (cat.color === color) return;
		try {
			await invoke('set_category_color', { id: cat.id, color });
			await reloadCats();
			toast.success(`Warna kategori "${cat.name}" diubah`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function handleAddTask(text: string, catId: number, description?: string) {
		await invoke('add_task', { text, categoryId: catId, projectId: activeProject, description });
		await refresh();
		toast.success('Todo berhasil ditambahkan');
		if (activeCat !== null && activeCat !== catId) {
			activeCat = catId;
			localStorage.setItem(CAT_KEY, String(catId));
		}
	}

	async function handleUpdateTask(id: number, text: string, catId: number, description?: string) {
		try {
			await invoke('update_task', { id, text, categoryId: catId, projectId: activeProject, description });
			await refresh();
			toast.success('Todo berhasil diperbarui');
		} catch (e) {
			alert(e);
		}
	}

	function handleToggleDone(t: Task) {
		const msg = t.done
			? `Kembalikan "${t.text}" ke belum selesai?`
			: `Tandai "${t.text}" selesai?`;
		askConfirm(msg, async () => {
			await invoke('set_done', { id: t.id, done: !t.done });
			await refresh();
			toast.success(t.done ? 'Todo dikembalikan ke belum selesai' : 'Todo ditandai selesai');
		}, 'default');
	}

	function handleSoftDelete(t: Task) {
		askConfirm(`Hapus "${t.text}"? Bisa dipulihkan di tab Dihapus.`, async () => {
			await invoke('soft_delete', { id: t.id });
			await refresh();
			toast.success('Todo dipindahkan ke Dihapus');
		}, 'destructive');
	}

	async function handleRestore(t: Task) {
		await invoke('restore', { id: t.id });
		await refresh();
		toast.success('Todo berhasil dipulihkan');
	}

	function handleHardDelete(t: Task) {
		askConfirm(`Hapus PERMANEN "${t.text}"? Tindakan ini tidak bisa dibatalkan.`, async () => {
			await invoke('hard_delete', { id: t.id });
			await refresh();
			toast.success('Todo berhasil dihapus permanen');
		}, 'destructive');
	}

	function handleBatchSetDone(ids: number[], done: boolean, onSuccess?: () => void) {
		if (ids.length === 0) return;
		const msg = done
			? `Tandai ${ids.length} tugas yang dipilih selesai?`
			: `Kembalikan ${ids.length} tugas yang dipilih ke daftar tugas?`;
		askConfirm(msg, async () => {
			await invoke('set_done_batch', { ids, done });
			await refresh();
			toast.success(done ? `${ids.length} tugas ditandai selesai` : `${ids.length} tugas dikembalikan ke daftar tugas`);
			onSuccess?.();
		}, 'default');
	}

	function handleBatchSoftDelete(ids: number[], onSuccess?: () => void) {
		if (ids.length === 0) return;
		askConfirm(`Hapus ${ids.length} todo terpilih? Bisa dipulihkan di tab Dihapus.`, async () => {
			await invoke('soft_delete_batch', { ids });
			await refresh();
			toast.success(`${ids.length} todo dipindahkan ke Dihapus`);
			onSuccess?.();
		}, 'destructive');
	}

	async function handleBatchRestore(ids: number[], onSuccess?: () => void) {
		if (ids.length === 0) return;
		await invoke('restore_batch', { ids });
		await refresh();
		toast.success(`${ids.length} todo berhasil dipulihkan`);
		onSuccess?.();
	}

	function handleBatchHardDelete(ids: number[], onSuccess?: () => void) {
		if (ids.length === 0) return;
		askConfirm(`Hapus PERMANEN ${ids.length} todo terpilih? Tindakan ini tidak bisa dibatalkan.`, async () => {
			await invoke('hard_delete_batch', { ids });
			await refresh();
			toast.success(`${ids.length} todo berhasil dihapus permanen`);
			onSuccess?.();
		}, 'destructive');
	}

	async function handleMoveTaskCategory(t: Task, catId: number) {
		if (catId === t.category_id) return;
		await invoke('update_task', {
			id: t.id,
			text: t.text,
			categoryId: catId,
			projectId: activeProject
		});
		await refresh();
		toast.success('Kategori todo berhasil diubah');
	}

	async function handleReorderTasks(newIds: number[]) {
		await invoke('reorder', { ids: newIds });
		await refresh();
	}

	// Persistent settings watcher
	$effect(() => {
		if (sortField) localStorage.setItem(SORT_FIELD_KEY, sortField);
	});
	$effect(() => {
		if (sortDir) localStorage.setItem(SORT_DIR_KEY, sortDir);
	});
	$effect(() => {
		if (activeCat !== null) {
			localStorage.setItem(CAT_KEY, String(activeCat));
		} else {
			localStorage.setItem(CAT_KEY, 'all');
		}
	});

	onMount(async () => {
		try {
			const status = await invoke<{ has_pin: boolean }>('get_lock_status');
			pinSet = status.has_pin;
		} catch {
			pinSet = false;
		}
		locked = true;

		await reloadProjects();
		exportDirectory =
			localStorage.getItem(EXPORT_DIR_KEY) ??
			(await invoke<string>('default_export_dir'));

		const savedProject = localStorage.getItem(PROJECT_KEY);
		const initialProject =
			projects.find((p) => String(p.id) === savedProject)?.id ?? projects[0]?.id ?? null;
		if (initialProject !== null) activeProject = initialProject;

		const savedSortField = localStorage.getItem(SORT_FIELD_KEY);
		if (savedSortField && ['cat_pos', 'pos', 'category', 'id', 'text'].includes(savedSortField)) {
			sortField = savedSortField as SortField;
		}
		const savedSortDir = localStorage.getItem(SORT_DIR_KEY);
		if (savedSortDir === 'asc' || savedSortDir === 'desc') {
			sortDir = savedSortDir as SortDir;
		}
	});
</script>

<LockOverlay bind:locked bind:pinSet />

{#if !locked}
	{#if page === 'home'}
		<ProjectsView
			{projects}
			{dark}
			onPickProject={pickProject}
			onOpenProjectDialog={openProjectDialog}
			onDeleteProject={deleteProject}
			onSetProjectStatus={setProjectStatus}
			onReorderProjects={handleReorderProjects}
			onLockApp={lockApp}
			onToggleTheme={toggleTheme}
			onOpenAbout={openAbout}
		/>
	{:else if page === 'categories'}
		<CategoriesView
			{cats}
			{tasks}
			{activeProjectItem}
			onBack={() => (page = 'tasks')}
			onOpenAddCategory={() => openCategoryDialog('add')}
			onOpenRenameCategory={(c) => openCategoryDialog('rename', c)}
			onDeleteCategory={deleteCategory}
			onMoveCategory={moveCategory}
			onReorderCategories={handleReorderCategories}
			onSetColorQuick={setCatColorQuick}
		/>
	{:else if page === 'about'}
		<AboutView onBack={() => (page = prevPage === 'about' ? 'tasks' : prevPage)} />
	{:else}
		<TasksView
			{tasks}
			{cats}
			{activeProjectItem}
			bind:activeCat
			bind:newCat
			bind:tab
			bind:sortField
			bind:sortDir
			{dark}
			onBackToProjects={() => (page = 'home')}
			onOpenProjectPicker={() => (projectPickerOpen = true)}
			onOpenCategoriesView={() => (page = 'categories')}
			onOpenRenameCategory={() => openCategoryDialog('rename')}
			onDeleteCategory={() => deleteCategory()}
			onOpenAddCategory={() => openCategoryDialog('add')}
			onLockApp={lockApp}
			onToggleTheme={toggleTheme}
			onOpenAbout={openAbout}
			onOpenImportDialog={() => (importDialogOpen = true)}
			onOpenExportDialog={() => (exportDialogOpen = true)}
			onAddTask={handleAddTask}
			onUpdateTask={handleUpdateTask}
			onToggleDone={handleToggleDone}
			onSetDoneBatch={handleBatchSetDone}
			onSoftDelete={handleSoftDelete}
			onSoftDeleteBatch={handleBatchSoftDelete}
			onRestore={handleRestore}
			onRestoreBatch={handleBatchRestore}
			onHardDelete={handleHardDelete}
			onHardDeleteBatch={handleBatchHardDelete}
			onMoveTaskCategory={(t) => (moveTask = t)}
			onReorderTasks={handleReorderTasks}
		/>
	{/if}
{/if}

<!-- Modals & Dialogs -->
<ExportDialog bind:open={exportDialogOpen} bind:exportDirectory />
<ImportDialog
	bind:open={importDialogOpen}
	onSuccess={async () => {
		await reloadProjects();
		if (activeProject !== null) {
			await reloadCats();
			await refresh();
		}
	}}
/>
<ProjectDialog
	bind:mode={projectDialogMode}
	project={editingProject}
	onSaved={async (id) => {
		await reloadProjects();
		if (projectDialogMode === 'add' && id) {
			await pickProject(id);
		}
	}}
/>
<ProjectPickerDialog
	bind:open={projectPickerOpen}
	{projects}
	{activeProject}
	onSelect={pickProject}
/>
<CategoryDialog
	bind:mode={categoryDialogMode}
	category={editingCategory}
	projectId={activeProject}
	onSaved={async (id) => {
		await reloadCats();
		if (categoryDialogMode === 'add' && id && page === 'tasks') {
			activeCat = id;
			newCat = id;
		}
	}}
/>
<MoveTaskDialog
	bind:task={moveTask}
	{cats}
	onMove={handleMoveTaskCategory}
/>
<ConfirmDialog bind:confirmState />
<Toaster closeButton theme={dark ? 'dark' : 'light'} position="top-center" />
