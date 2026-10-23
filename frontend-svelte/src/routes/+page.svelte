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
		AUTOLOCK_KEY,
		type Task,
		type TaskStatus,
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
	import SecuritySettingsDialog from '#lib/components/dialogs/SecuritySettingsDialog.svelte';
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
	let activeCats = $state<number[]>([]);
	let newCat = $state<number | null>(null);
	let sortField = $state<SortField>('cat_pos');
	let sortDir = $state<SortDir>('asc');
	let exportDirectory = $state('');

	// Dialog & Action State
	let locked = $state(true);
	let pinSet = $state(false);
	let autoLockDelay = $state<string>('never');
	let securityDialogOpen = $state(false);
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

	function openSecuritySettings() {
		securityDialogOpen = true;
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
		activeCats = [];
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
		const current = activeCats.length === 1 ? activeCats[0] : null;
		editingCategory = cat ?? (mode === 'rename' ? (cats.find(c => c.id === current) ?? cats[0] ?? null) : null);
		categoryDialogMode = mode;
	}

	function deleteCategory(target?: Category) {
		const current = activeCats.length === 1 ? activeCats[0] : null;
		const cur = target ?? cats.find(c => c.id === current);
		if (!cur) return;
		askConfirm(`Hapus kategori "${cur.name}"?`, async () => {
			try {
				await invoke('delete_category', { id: cur.id, projectId: activeProject });
				await reloadCats();
				if (!cats.some((c) => c.id === cur.id)) {
					activeCats = activeCats.filter((id) => id !== cur.id);
					localStorage.setItem(CAT_KEY, activeCats.length ? JSON.stringify(activeCats) : 'all');
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
		if (activeCats.length > 0 && !activeCats.includes(catId)) {
			activeCats = [...activeCats, catId];
			localStorage.setItem(CAT_KEY, JSON.stringify(activeCats));
		}
	}

	async function handleDuplicateTask(t: Task) {
		try {
			await invoke('add_task', {
				text: t.text,
				categoryId: t.category_id,
				projectId: activeProject,
				description: t.description ?? ''
			});
			await refresh();
			toast.success('Todo berhasil diduplikat');
		} catch (e) {
			toast.error(String(e));
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

	async function handleSetTaskStatus(id: number, status: TaskStatus) {
		await invoke('set_task_status', { id, status });
		await refresh();
		const label = status === 'inprogress' ? 'In Progress' : status === 'done' ? 'Selesai' : 'Todo';
		toast.success(`Status diubah ke ${label}`);
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
			? `Tandai ${ids.length} todo yang dipilih selesai?`
			: `Kembalikan ${ids.length} todo yang dipilih ke daftar todo?`;
		askConfirm(msg, async () => {
			await invoke('set_done_batch', { ids, done });
			await refresh();
			toast.success(done ? `${ids.length} todo ditandai selesai` : `${ids.length} todo dikembalikan ke daftar todo`);
			onSuccess?.();
		}, 'default');
	}

	function handleBatchSetTaskStatus(ids: number[], status: TaskStatus, onSuccess?: () => void) {
		if (ids.length === 0) return;
		const label = status === 'inprogress' ? 'In Progress' : status === 'done' ? 'Selesai' : 'Todo';
		askConfirm(`Ubah status ${ids.length} todo terpilih menjadi "${label}"?`, async () => {
			await invoke('set_task_status_batch', { ids, status });
			await refresh();
			toast.success(`${ids.length} todo diubah ke ${label}`);
			onSuccess?.();
		}, 'default', 'Ubah Status');
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

	function handleBatchRestore(ids: number[], onSuccess?: () => void) {
		if (ids.length === 0) return;
		askConfirm(`Pulihkan ${ids.length} todo terpilih?`, async () => {
			await invoke('restore_batch', { ids });
			await refresh();
			toast.success(`${ids.length} todo berhasil dipulihkan`);
			onSuccess?.();
		}, 'default', 'Pulihkan');
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

	function handleBatchDuplicate(ids: number[], onSuccess?: () => void) {
		if (ids.length === 0) return;
		askConfirm(`Duplikat ${ids.length} todo terpilih?`, async () => {
			try {
				await invoke('duplicate_tasks_batch', { ids });
				await refresh();
				toast.success(`${ids.length} todo berhasil diduplikat`);
				onSuccess?.();
			} catch (e) {
				toast.error(String(e));
			}
		}, 'default', 'Duplikat');
	}

	async function handleBatchSetCategory(ids: number[], categoryId: number, onSuccess?: () => void) {
		if (ids.length === 0) return;
		try {
			await invoke('set_category_batch', { ids, categoryId, projectId: activeProject });
			await refresh();
			const cat = cats.find((c) => c.id === categoryId);
			toast.success(`${ids.length} todo berhasil dipindahkan ke ${cat?.name ?? 'kategori baru'}`);
			onSuccess?.();
		} catch (e) {
			toast.error(String(e));
		}
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
		localStorage.setItem(CAT_KEY, activeCats.length ? JSON.stringify(activeCats) : 'all');
	});

	onMount(() => {
		let unlistenFocus: (() => void) | null = null;
		let hiddenTimestamp: number | null = null;
		let autoLockTimeoutId: any = null;

		function clearAutoLockTimer() {
			if (autoLockTimeoutId) {
				clearTimeout(autoLockTimeoutId);
				autoLockTimeoutId = null;
			}
		}

		function checkAndLock() {
			if (autoLockDelay === 'never' || !pinSet || locked) return;
			locked = true;
			hiddenTimestamp = null;
			clearAutoLockTimer();
		}

		function onWindowHidden() {
			if (autoLockDelay === 'never' || !pinSet || locked) return;

			if (autoLockDelay === '0') {
				checkAndLock();
				return;
			}

			const delaySec = parseInt(autoLockDelay, 10);
			if (isNaN(delaySec) || delaySec <= 0) return;

			hiddenTimestamp = Date.now();
			clearAutoLockTimer();
			autoLockTimeoutId = setTimeout(() => {
				checkAndLock();
			}, delaySec * 1000);
		}

		function onWindowVisible() {
			clearAutoLockTimer();
			if (hiddenTimestamp !== null && autoLockDelay !== 'never' && pinSet && !locked) {
				const delaySec = parseInt(autoLockDelay, 10);
				if (!isNaN(delaySec) && delaySec >= 0) {
					const elapsedSec = (Date.now() - hiddenTimestamp) / 1000;
					if (elapsedSec >= delaySec) {
						checkAndLock();
					}
				}
			}
			hiddenTimestamp = null;
		}

		const handleVisibility = () => {
			if (document.hidden || document.visibilityState === 'hidden') {
				onWindowHidden();
			} else {
				onWindowVisible();
			}
		};
		document.addEventListener('visibilitychange', handleVisibility);

		// Window blur/focus listeners
		const handleBlur = () => {
			if (autoLockDelay !== '0') {
				onWindowHidden();
			}
		};
		const handleFocus = () => {
			onWindowVisible();
		};
		window.addEventListener('blur', handleBlur);
		window.addEventListener('focus', handleFocus);

		// Also listen to Tauri window focus change if available
		import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
			const appWindow = getCurrentWindow();
			appWindow.onFocusChanged(({ payload: focused }) => {
				if (!focused) {
					appWindow.isMinimized().then((minimized) => {
						if (minimized) {
							onWindowHidden();
						}
					}).catch(() => {});
				} else {
					onWindowVisible();
				}
			}).then((unsub) => {
				unlistenFocus = unsub;
			}).catch(() => {});
		}).catch(() => {});

		(async () => {
			try {
				const status = await invoke<{ has_pin: boolean }>('get_lock_status');
				pinSet = status.has_pin;
			} catch {
				pinSet = false;
			}
			locked = true;

			autoLockDelay = localStorage.getItem(AUTOLOCK_KEY) ?? 'never';

			await reloadProjects();
			exportDirectory =
				localStorage.getItem(EXPORT_DIR_KEY) ??
				localStorage.getItem('taskmini.exportDirectory') ??
				(await invoke<string>('default_export_dir'));

			const savedProject = localStorage.getItem(PROJECT_KEY) ?? localStorage.getItem('taskmini.project');
			const initialProject =
				projects.find((p) => String(p.id) === savedProject)?.id ?? projects[0]?.id ?? null;
			if (initialProject !== null) activeProject = initialProject;

			const savedSortField = localStorage.getItem(SORT_FIELD_KEY) ?? localStorage.getItem('taskmini.sortField');
			if (savedSortField && ['cat_pos', 'pos', 'category', 'id', 'text'].includes(savedSortField)) {
				sortField = savedSortField as SortField;
			}
			const savedSortDir = localStorage.getItem(SORT_DIR_KEY) ?? localStorage.getItem('taskmini.sortDir');
			if (savedSortDir === 'asc' || savedSortDir === 'desc') {
				sortDir = savedSortDir as SortDir;
			}
		})();

		return () => {
			clearAutoLockTimer();
			document.removeEventListener('visibilitychange', handleVisibility);
			window.removeEventListener('blur', handleBlur);
			window.removeEventListener('focus', handleFocus);
			unlistenFocus?.();
		};
	});
</script>

<div
	class="h-full w-full"
	inert={locked ? true : undefined}
	aria-hidden={locked ? 'true' : undefined}
>
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
			onOpenSecuritySettings={openSecuritySettings}
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
		<AboutView
			onBack={() => (page = prevPage === 'about' ? 'tasks' : prevPage)}
			bind:autoLockDelay
			{pinSet}
			onOpenSecuritySettings={openSecuritySettings}
			onLockApp={lockApp}
		/>
	{:else}
		<TasksView
			{tasks}
			{cats}
			{activeProjectItem}
			bind:activeCats
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
			onOpenSecuritySettings={openSecuritySettings}
			onOpenImportDialog={() => (importDialogOpen = true)}
			onOpenExportDialog={() => (exportDialogOpen = true)}
			onAddTask={handleAddTask}
			onUpdateTask={handleUpdateTask}
			onDuplicateTask={handleDuplicateTask}
			onToggleDone={handleToggleDone}
			onSetTaskStatus={handleSetTaskStatus}
			onSetTaskStatusBatch={handleBatchSetTaskStatus}
			onSetDoneBatch={handleBatchSetDone}
			onSoftDelete={handleSoftDelete}
			onSoftDeleteBatch={handleBatchSoftDelete}
			onRestore={handleRestore}
			onRestoreBatch={handleBatchRestore}
			onHardDelete={handleHardDelete}
			onHardDeleteBatch={handleBatchHardDelete}
			onDuplicateBatch={handleBatchDuplicate}
			onSetCategoryBatch={handleBatchSetCategory}
			onMoveTaskCategory={(t) => (moveTask = t)}
			onReorderTasks={handleReorderTasks}
			onOpenEditProject={() => {
				editingProject = activeProjectItem ?? null;
				projectDialogMode = 'edit';
			}}
			onSetProjectStatus={setProjectStatus}
			onRefreshTasks={refresh}
		/>
	{/if}
</div>

<LockOverlay bind:locked bind:pinSet />

<SecuritySettingsDialog
	bind:open={securityDialogOpen}
	bind:autoLockDelay
	bind:pinSet
	onLockNow={lockApp}
/>

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
			activeCats = [...activeCats, id];
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
