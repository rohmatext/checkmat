<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';
	import { Check, RotateCcw, Trash2, Undo2, Pencil, Plus, X, Sun, Moon, GripVertical, Download, ArrowLeft, FolderOpen, Upload, Lock } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Tabs from '#lib/components/ui/tabs/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import Toaster from '#lib/components/ui/sonner/sonner.svelte';
	import { toast as sonnerToast } from 'svelte-sonner';
	import { cn } from '#lib/utils';

	type Task = {
		id: number;
		text: string;
		done: boolean;
		deleted: boolean;
		pos: number;
		category_id: number;
		project_id: number;
	};
	type Category = { id: number; name: string; color: string };
	type Project = { id: number; name: string; description: string };
	type Tab = 'active' | 'done' | 'deleted';
	type Page = 'home' | 'tasks';

	const PALETTE = ['red', 'orange', 'amber', 'green', 'cyan', 'blue', 'violet', 'pink'];
	// gaya label GitHub: bg lembut + teks & border sewarna; mode dark pakai shade 950/400/900
	const TONE: Record<string, { badge: string; solid: string }> = {
		red: { badge: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-950 dark:text-red-400 dark:border-red-900', solid: 'bg-red-500' },
		orange: { badge: 'bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950 dark:text-orange-400 dark:border-orange-900', solid: 'bg-orange-500' },
		amber: { badge: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950 dark:text-amber-400 dark:border-amber-900', solid: 'bg-amber-500' },
		green: { badge: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-950 dark:text-green-400 dark:border-green-900', solid: 'bg-green-500' },
		cyan: { badge: 'bg-cyan-50 text-cyan-700 border-cyan-200 dark:bg-cyan-950 dark:text-cyan-400 dark:border-cyan-900', solid: 'bg-cyan-500' },
		blue: { badge: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950 dark:text-blue-400 dark:border-blue-900', solid: 'bg-blue-500' },
		violet: { badge: 'bg-violet-50 text-violet-700 border-violet-200 dark:bg-violet-950 dark:text-violet-400 dark:border-violet-900', solid: 'bg-violet-500' },
		pink: { badge: 'bg-pink-50 text-pink-700 border-pink-200 dark:bg-pink-950 dark:text-pink-400 dark:border-pink-900', solid: 'bg-pink-500' }
	};

	const CAT_KEY = 'taskmini.cat';
	const PROJECT_KEY = 'taskmini.project';

	let tasks = $state<Task[]>([]);
	let cats = $state<Category[]>([]);
	let projects = $state<Project[]>([]);
	let activeProject = $state<number | null>(null);
	let page = $state<Page>('home');
	let projectPickerOpen = $state(false);
	let exportDialogOpen = $state(false);
	let importDialogOpen = $state(false);
	let exportDirectory = $state('');
	let exportFileName = $state('');
	let importProcessed = $state(0);
	let importTotal = $state(0);
	let importBusy = $state(false);
	let fileActionError = $state('');
	let tab = $state<Tab>('active');
	let activeCat = $state<number | null>(null);
	let newCat = $state<number | null>(null);
	let dragRows = $state<Task[] | null>(null);
	let newText = $state('');
	let editId = $state<number | null>(null);
	let editVal = $state('');
	let dragId = $state<number | null>(null);
	let overIdx = $state<number | null>(null);
	let catDialog = $state<'add' | 'rename' | null>(null);
	let catName = $state('');
	let catColor = $state(PALETTE[0]);
	let catErr = $state('');
	let projectDialog = $state<'add' | 'edit' | null>(null);
	let projectName = $state('');
	let projectDescription = $state('');
	let editingProjectId = $state<number | null>(null);
	let projectErr = $state('');
	let confirmState = $state<{ msg: string; run: () => Promise<void> } | null>(null);
	let moveTask = $state<Task | null>(null);

	// proteksi PIN
	let locked = $state(true);
	let pinSet = $state(false); // apakah PIN sudah pernah dibuat
	let lockStage = $state<'verify' | 'setup' | 'confirm'>('verify');
	let lockInput = $state('');
	let lockConfirmInput = $state('');
	let lockError = $state('');
	let lockBusy = $state(false);


	const byId = (id: number | null) => cats.find((c) => c.id === id);
	const activeProjectItem = $derived(projects.find((p) => p.id === activeProject));
	const inTab = (t: Task, k: Tab) =>
		k === 'active' ? !t.done && !t.deleted : k === 'done' ? t.done && !t.deleted : t.deleted;

	const badgeCls = (id: number | null) =>
		TONE[byId(id)?.color ?? '']?.badge ?? 'bg-muted text-muted-foreground border-border';
	const solidCls = (name: string) => TONE[name]?.solid ?? 'bg-gray-400';

	let dark = $state(typeof document !== 'undefined' && document.documentElement.classList.contains('dark'));
	function toggleTheme() {
		dark = !dark;
		document.documentElement.classList.toggle('dark', dark);
		localStorage.setItem('taskmini.theme', dark ? 'dark' : 'light');
	}

	function showSuccess(msg: string) {
		sonnerToast.success(msg);
	}

	function showError(msg: string) {
		sonnerToast.error(msg);
	}

	function lockApp() {
		lockInput = '';
		lockConfirmInput = '';
		lockError = '';
		lockStage = pinSet ? 'verify' : 'setup';
		locked = true;
	}

	async function submitLock() {
		if (lockBusy) return;
		lockError = '';
		if (lockStage === 'setup') {
			if (lockInput.length < 4) {
				lockError = 'PIN minimal 4 digit.';
				return;
			}
			lockConfirmInput = '';
			lockStage = 'confirm';
			return;
		}
		if (lockStage === 'confirm') {
			if (lockInput !== lockConfirmInput) {
				lockError = 'PIN tidak sama, coba lagi.';
				lockConfirmInput = '';
				return;
			}
			lockBusy = true;
			try {
				await invoke('set_pin', { pin: lockInput });
				pinSet = true;
				locked = false;
				showSuccess('PIN berhasil dibuat');
			} catch (e) {
				lockError = String(e);
			} finally {
				lockBusy = false;
			}
			return;
		}
		// verify
		lockBusy = true;
		try {
			const ok = await invoke<boolean>('verify_pin', { pin: lockInput });
			if (ok) {
				locked = false;
			} else {
				lockError = 'PIN salah.';
				lockInput = '';
			}
		} catch (e) {
			lockError = String(e);
		} finally {
			lockBusy = false;
		}
	}

	async function resetPin() {
		try {
			await invoke('clear_pin');
			pinSet = false;
			lockStage = 'setup';
			lockInput = '';
			lockConfirmInput = '';
			lockError = '';
			showSuccess('PIN direset. Buat PIN baru.');
		} catch (e) {
			showError(String(e));
		}
	}

	function timestampFileName() {
		const d = new Date();
		const part = (v: number) => String(v).padStart(2, '0');
		return `${part(d.getFullYear() % 100)}${part(d.getMonth() + 1)}${part(d.getDate())}${part(d.getHours())}${part(d.getMinutes())}${part(d.getSeconds())}_taskmini`;
	}

	function openExportDialog() {
		exportFileName = timestampFileName();
		fileActionError = '';
		exportDialogOpen = true;
	}

	async function chooseExportDirectory() {
		try {
			const selected = await open({
				directory: true,
				multiple: false,
				title: 'Pilih folder untuk menyimpan CSV'
			});
			if (typeof selected === 'string') {
				exportDirectory = selected;
				localStorage.setItem('taskmini.exportDirectory', selected);
			}
		} catch (e) {
			fileActionError = String(e);
		}
	}

	async function exportCsv() {
		const name = exportFileName.trim().replace(/[<>:"/\\|?*]/g, '_');
		if (!exportDirectory || !name) {
			fileActionError = 'Pilih folder dan isi nama file.';
			return;
		}
		try {
			const path = `${exportDirectory.replace(/[\\/]+$/, '')}\\${name}.csv`;
			const savedPath = await invoke<string>('export_csv', { path });
			exportDialogOpen = false;
			showSuccess(`CSV disimpan ke ${savedPath}`);
		} catch (e) {
			fileActionError = String(e);
		}
	}

	async function saveCsvTemplate() {
		try {
			const selected = await open({
				directory: true,
				multiple: false,
				title: 'Pilih folder untuk menyimpan template CSV'
			});
			if (typeof selected !== 'string') return;
			const path = `${selected.replace(/[\\/]+$/, '')}\\template_taskmini.csv`;
			const savedPath = await invoke<string>('write_csv_template', { path });
			showSuccess(`Template disimpan ke ${savedPath}`);
		} catch (e) {
			fileActionError = String(e);
		}
	}

	function openImportDialog() {
		fileActionError = '';
		importDialogOpen = true;
	}

	async function importCsv() {
		fileActionError = '';
		try {
			const selected = await open({
				multiple: false,
				title: 'Pilih file CSV untuk diimport',
				filters: [{ name: 'CSV', extensions: ['csv'] }]
			});
			if (typeof selected !== 'string') return;
			importProcessed = 0;
			importTotal = 0;
			importBusy = true;
			let unlisten: (() => void) | undefined;
			try {
			unlisten = await listen<{ processed: number; total: number }>(
				'import-progress',
				(event) => {
					importProcessed = event.payload.processed;
					importTotal = event.payload.total;
				}
			);
			const count = await invoke<number>('import_csv', { path: selected });
			importProcessed = count;
			importTotal = count;
			await reloadProjects();
			if (activeProject !== null) {
				await reloadCats();
				await refresh();
			}
			importDialogOpen = false;
			showSuccess(`${count} todo berhasil diimport`);
			} catch (e) {
				fileActionError = String(e);
				showError(`Import gagal: ${String(e)}`);
			} finally {
				unlisten?.();
				importBusy = false;
			}
		} catch (e) {
			fileActionError = String(e);
		}
	}

	let rows = $derived(
		tasks.filter((t) => (activeCat === null || t.category_id === activeCat) && inTab(t, tab))
	);
	let curCat = $derived(byId(activeCat));
	let view = $derived(dragRows ?? rows);

	const count = (k: Tab) =>
		tasks.filter((t) => (activeCat === null || t.category_id === activeCat) && inTab(t, k)).length;

	function pickCat(v: string) {
		activeCat = v === 'all' ? null : Number(v);
		localStorage.setItem(CAT_KEY, v);
		if (activeCat !== null) newCat = activeCat;
	}

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
	async function pickProject(value: string | number) {
		const id = Number(value);
		if (!Number.isInteger(id) || !projects.some((p) => p.id === id)) return;
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
		projectName = mode === 'edit' ? (project?.name ?? '') : '';
		projectDescription = mode === 'edit' ? (project?.description ?? '') : '';
		editingProjectId = mode === 'edit' ? (project?.id ?? null) : null;
		projectErr = '';
		projectDialog = mode;
	}

	async function submitProject() {
		const name = projectName.trim();
		if (!name) return;
		try {
			if (projectDialog === 'add') {
				await invoke<number>('add_project', { name, description: projectDescription });
				await reloadProjects();
			} else if (editingProjectId !== null) {
				await invoke('update_project', {
					id: editingProjectId,
					name,
					description: projectDescription
				});
				await reloadProjects();
			}
			showSuccess(projectDialog === 'add' ? 'Project berhasil ditambahkan' : 'Project berhasil diperbarui');
			projectDialog = null;
		} catch (e) {
			projectErr = String(e);
		}
	}

	function deleteProject(selected: Project) {
		askConfirm(`Hapus project "${selected.name}"?`, async () => {
			try {
				await invoke('delete_project', { id: selected.id });
				await reloadProjects();
				if (activeProject === selected.id) {
					activeProject = null;
					tasks = [];
					cats = [];
					localStorage.removeItem(PROJECT_KEY);
				}
				showSuccess(`Project "${selected.name}" berhasil dihapus`);
			} catch (e) {
				showError(String(e));
			}
		});
	}

	function openCatDialog(mode: 'add' | 'rename') {
		catName = mode === 'rename' ? (curCat?.name ?? '') : '';
		const c = mode === 'rename' ? curCat?.color : '';
		catColor = c && TONE[c] ? c : PALETTE[0];
		catErr = '';
		catDialog = mode;
	}

	async function submitCat() {
		const name = catName.trim();
		if (!name) return;
		try {
			if (catDialog === 'add') {
				if (activeProject === null) return;
				const id = await invoke<number>('add_category', {
					name,
					color: catColor,
					projectId: activeProject
				});
				await reloadCats();
				pickCat(String(id));
				showSuccess(`Kategori "${name}" berhasil ditambahkan`);
			} else {
				const cur = curCat;
				if (!cur) return (catDialog = null);
				let changed = false;
				if (name !== cur.name) {
					await invoke('rename_category', { id: cur.id, name, projectId: activeProject });
					changed = true;
				}
				if (catColor !== cur.color) {
					await invoke('set_category_color', { id: cur.id, color: catColor });
					changed = true;
				}
				await reloadCats();
				if (changed) showSuccess(`Kategori "${name}" berhasil diperbarui`);
			}
			catDialog = null;
		} catch (e) {
			catErr = String(e);
		}
	}

	function askConfirm(msg: string, run: () => Promise<void>) {
		confirmState = { msg, run };
	}

	function deleteCat() {
		const cur = curCat;
		if (!cur) return;
		askConfirm(`Hapus kategori "${cur.name}"?`, async () => {
			await invoke('delete_category', { id: cur.id, projectId: activeProject });
			await reloadCats();
			if (!cats.some((c) => c.id === activeCat)) pickCat(cats.length ? String(cats[0].id) : 'all');
			if (newCat === null || !cats.some((c) => c.id === newCat)) newCat = cats[0]?.id ?? null;
			await refresh();
			showSuccess(`Kategori "${cur.name}" berhasil dihapus`);
		});
	}

	async function add() {
		const text = newText.trim();
		if (!text) return;
		const catId = newCat ?? cats[0]?.id;
		if (catId == null) {
			alert('Belum ada kategori.');
			return;
		}
		newText = '';
		tab = 'active';
		await invoke('add_task', { text, categoryId: catId, projectId: activeProject });
		await refresh();
		showSuccess('Todo berhasil ditambahkan');
		// pindah ke kategori task baru; kalau lagi di "Semua kategori" tetap
		if (activeCat !== null && activeCat !== catId) pickCat(String(catId));
	}

	async function moveCat(t: Task, id: number) {
		if (id === t.category_id) return;
		await invoke('update_task', {
			id: t.id,
			text: t.text,
			categoryId: id,
			projectId: activeProject
		});
		await refresh();
		showSuccess('Kategori todo berhasil diubah');
	}

	function startEdit(t: Task) {
		if (t.deleted) return;
		editId = t.id;
		editVal = t.text;
	}

	async function finishEdit(save: boolean) {
		const id = editId;
		const t = tasks.find((x) => x.id === id);
		editId = null;
		if (!t) return;
		const val = editVal.trim();
		if (save && val && val !== t.text) {
			try {
				await invoke('update_task', {
					id: t.id,
					text: val,
					categoryId: t.category_id,
					projectId: activeProject
				});
				await refresh();
				showSuccess('Todo berhasil diperbarui');
			} catch (e) {
				alert(e);
			}
		}
	}

	const onEditKey = (e: KeyboardEvent) => {
		if (e.key === 'Enter') finishEdit(true);
		else if (e.key === 'Escape') finishEdit(false);
	};

	function toggleDone(t: Task) {
		const msg = t.done
			? `Kembalikan "${t.text}" ke belum selesai?`
			: `Tandai "${t.text}" selesai?`;
		askConfirm(msg, async () => {
			await invoke('set_done', { id: t.id, done: !t.done });
			await refresh();
			showSuccess(t.done ? 'Todo dikembalikan ke belum selesai' : 'Todo ditandai selesai');
		});
	}

	function softDelete(t: Task) {
		askConfirm(`Hapus "${t.text}"? Bisa dipulihkan di tab Dihapus.`, async () => {
			await invoke('soft_delete', { id: t.id });
			await refresh();
			showSuccess('Todo dipindahkan ke Dihapus');
		});
	}

	async function restore(t: Task) {
		await invoke('restore', { id: t.id });
		await refresh();
		showSuccess('Todo berhasil dipulihkan');
	}

	function hardDelete(t: Task) {
		askConfirm(`Hapus PERMANEN "${t.text}"? Tindakan ini tidak bisa dibatalkan.`, async () => {
			await invoke('hard_delete', { id: t.id });
			await refresh();
			showSuccess('Todo berhasil dihapus permanen');
		});
	}

	/* ----- drag reorder ----- */
	function shift(idx: number) {
		const cur = dragRows ?? rows;
		const from = cur.findIndex((r) => r.id === dragId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragRows = next;
		overIdx = idx > from ? idx - 1 : idx;
	}

	function onDragStart(e: DragEvent, t: Task) {
		if (tab === 'deleted') {
			e.preventDefault();
			return;
		}
		dragId = t.id;
		dragRows = rows;
		overIdx = null;
		e.dataTransfer!.effectAllowed = 'move';
		e.dataTransfer!.setData('text/plain', String(t.id));
	}

	function onRowDragOver(e: DragEvent, i: number) {
		if (dragId === null) return;
		e.preventDefault();
		e.dataTransfer!.dropEffect = 'move';
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		shift(i + (e.clientY > rect.top + rect.height / 2 ? 1 : 0));
	}

	function onMainDragOver(e: DragEvent) {
		// hanya kena padding container, bukan event bubble dari row
		if (dragId === null || e.target !== e.currentTarget) return;
		e.preventDefault();
		e.dataTransfer!.dropEffect = 'move';
		shift(view.length);
	}

	async function onDrop(e: DragEvent) {
		e.preventDefault();
		const order = dragRows;
		dragId = null;
		overIdx = null;
		dragRows = null;
		if (!order) return;
		await invoke('reorder', { ids: order.map((r) => r.id) });
		await refresh();
		showSuccess('Urutan todo berhasil disimpan');
	}

	function onDragEnd() {
		// drop di luar list / ESC: batalkan preview tanpa menyimpan
		dragId = null;
		overIdx = null;
		dragRows = null;
	}

	onMount(async () => {
		try {
			const status = await invoke<{ has_pin: boolean }>('get_lock_status');
			pinSet = status.has_pin;
		} catch {
			pinSet = false;
		}
		lockStage = pinSet ? 'verify' : 'setup';
		locked = true;
		await reloadProjects();
		exportDirectory =
			localStorage.getItem('taskmini.exportDirectory') ??
			(await invoke<string>('default_export_dir'));
		const savedProject = localStorage.getItem(PROJECT_KEY);
		const initialProject =
			projects.find((p) => String(p.id) === savedProject)?.id ?? projects[0]?.id ?? null;
		if (initialProject !== null) activeProject = initialProject;
	});

	const tabs: { k: Tab; label: string }[] = [
		{ k: 'active', label: 'Task' },
		{ k: 'done', label: 'Selesai' },
		{ k: 'deleted', label: 'Dihapus' }
	];
</script>

{#if locked}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/95 p-6 backdrop-blur">
		<div class="flex w-full max-w-xs flex-col items-center gap-4">
			<div class="flex size-14 items-center justify-center rounded-full border bg-card">
				<Lock class="size-7 text-primary" />
			</div>
			{#if lockStage === 'setup'}
				<div class="text-center">
					<h2 class="text-base font-bold">Buat PIN</h2>
					<p class="text-xs text-muted-foreground">Buat PIN 4-6 digit untuk mengunci aplikasi.</p>
				</div>
			{:else if lockStage === 'confirm'}
				<div class="text-center">
					<h2 class="text-base font-bold">Ulangi PIN</h2>
					<p class="text-xs text-muted-foreground">Masukkan kembali PIN untuk memastikan.</p>
				</div>
			{:else}
				<div class="text-center">
					<h2 class="text-base font-bold">Aplikasi terkunci</h2>
					<p class="text-xs text-muted-foreground">Masukkan PIN untuk membuka.</p>
				</div>
			{/if}
			<div class="w-full space-y-2">
				<!-- svelte-ignore a11y_autofocus -->
				<Input
					type="password"
					inputmode="numeric"
					autocomplete="off"
					maxlength={6}
					autofocus
					bind:value={lockInput}
					onkeydown={(e) => e.key === 'Enter' && submitLock()}
					class="h-10 text-center text-lg tracking-[0.5em]"
					placeholder="••••"
				/>
				{#if lockStage === 'confirm'}
					<!-- svelte-ignore a11y_autofocus -->
					<Input
						type="password"
						inputmode="numeric"
						autocomplete="off"
						maxlength={6}
						bind:value={lockConfirmInput}
						onkeydown={(e) => e.key === 'Enter' && submitLock()}
						class="h-10 text-center text-lg tracking-[0.5em]"
						placeholder="••••"
					/>
				{/if}
			</div>
			{#if lockError}
				<p class="text-xs text-destructive">{lockError}</p>
			{/if}
			<div class="flex w-full flex-col gap-2">
				<Button class="w-full" disabled={lockBusy} onclick={submitLock}>
					{lockStage === 'setup' ? 'Lanjut' : lockStage === 'confirm' ? 'Simpan PIN' : 'Buka'}
				</Button>
				{#if lockStage === 'verify' && pinSet}
					<button
						type="button"
						class="text-xs text-muted-foreground underline-offset-2 hover:underline"
						onclick={resetPin}
					>
						Lupa PIN? Reset
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}

{#if page === 'home'}
	<div class="flex h-full flex-col p-3">
		<header class="flex items-center justify-between gap-2 pb-3">
			<div class="min-w-0">
				<h1 class="text-lg font-bold">Project</h1>
				<p class="text-xs text-muted-foreground">Pilih project untuk membuka daftar tugas.</p>
			</div>
			<Button size="sm" onclick={() => openProjectDialog('add')}><Plus /> Project</Button>
		</header>
		<main class="flex-1 space-y-2 overflow-y-auto">
			{#if projects.length === 0}
				<div class="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
					Belum ada project. Buat project pertama untuk mulai mencatat todo.
				</div>
			{/if}
			{#each projects as project (project.id)}
				<div class="flex items-center gap-2 rounded-lg border bg-card p-3">
					<button
						type="button"
						class="flex min-w-0 flex-1 items-start gap-3 text-left"
						onclick={() => pickProject(project.id)}
					>
						<FolderOpen class="mt-0.5 size-5 shrink-0 text-primary" />
						<span class="min-w-0 flex-1">
							<span class="block truncate text-sm font-semibold">{project.name}</span>
							<span class="mt-0.5 line-clamp-2 block text-xs text-muted-foreground">
								{project.description || 'Belum ada deskripsi'}
							</span>
						</span>
					</button>
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="outline"
									size="icon-sm"
									title="Edit project"
									onclick={() => openProjectDialog('edit', project)}
								>
									<Pencil />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Edit project</Tooltip.Content>
					</Tooltip.Root>
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="outline"
									size="icon-sm"
									disabled={projects.length <= 1}
									title="Hapus project"
									class="hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
									onclick={() => deleteProject(project)}
								>
									<Trash2 />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Hapus project</Tooltip.Content>
					</Tooltip.Root>
				</div>
			{/each}
		</main>
		<footer class="flex justify-end gap-1.5 pt-2">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" title="Kunci aplikasi" onclick={lockApp}>
							<Lock />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Kunci aplikasi</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" onclick={toggleTheme}>
							{#if dark}<Sun />{:else}<Moon />{/if}
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>{dark ? 'Mode terang' : 'Mode gelap'}</Tooltip.Content>
			</Tooltip.Root>
		</footer>
	</div>
{:else}
<div class="flex h-full flex-col">
	<!-- project title and navigation -->
	<div class="flex items-center gap-1 px-2 pt-2">
		<Button variant="outline" size="icon-sm" title="Kembali ke project" onclick={() => (page = 'home')}>
			<ArrowLeft />
		</Button>
		<button
			type="button"
			class="min-w-0 flex-1 truncate px-2 text-left text-sm font-bold hover:text-primary"
			title="Pilih project lain"
			onclick={() => (projectPickerOpen = true)}
		>
			{activeProjectItem?.name ?? 'Pilih project'}
		</button>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={() => (projectPickerOpen = true)}>
						<FolderOpen />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Pilih project</Tooltip.Content>
		</Tooltip.Root>
	</div>

	<!-- filter kategori -->
	<div class="flex items-center gap-1 px-2 pt-2">
		<Select.Root
			type="single"
			value={activeCat === null ? 'all' : String(activeCat)}
			onValueChange={(v) => v && pickCat(v)}
		>
			<Select.Trigger class="h-8 min-w-0 flex-1 rounded-md font-semibold">
				<Select.Value placeholder="Semua kategori">
					{#snippet children()}
						{activeCat === null ? 'Semua kategori' : (byId(activeCat)?.name ?? 'Semua kategori')}
					{/snippet}
				</Select.Value>
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="all" label="Semua kategori" />
				{#each cats as c (c.id)}
					<Select.Item value={String(c.id)} label={c.name} />
				{/each}
			</Select.Content>
		</Select.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" disabled={activeCat === null} onclick={() => openCatDialog('rename')}>
						<Pencil />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Ganti nama kategori</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						variant="outline"
						size="icon-sm"
						disabled={activeCat === null}
						onclick={deleteCat}
						class="hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
					>
						<Trash2 />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Hapus kategori</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" title="Kunci aplikasi" onclick={lockApp}>
						<Lock />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Kunci aplikasi</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={toggleTheme}>
						{#if dark}<Sun />{:else}<Moon />{/if}
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>{dark ? 'Mode terang' : 'Mode gelap'}</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={openImportDialog}>
						<Upload />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Import CSV</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={openExportDialog}>
						<Download />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Export CSV</Tooltip.Content>
		</Tooltip.Root>
	</div>

	<!-- tabs -->
	<Tabs.Root value={tab} onValueChange={(v) => (tab = v as Tab)} class="gap-0 px-2 pt-2">
		<Tabs.List class="grid w-full grid-cols-3 gap-1 rounded-t-lg rounded-b-none bg-transparent p-0">
			{#each tabs as { k, label } (k)}
				{@const n = count(k)}
				<Tabs.Trigger
					value={k}
					class="cursor-pointer rounded-t-lg border border-b-0 border-border px-1.5 text-xs font-semibold text-muted-foreground data-[state=active]:bg-card data-[state=active]:text-primary data-[state=active]:shadow-[inset_0_-1px_0_var(--color-primary)]"
				>
					{label}
					{#if n > 0}
						<Badge variant={tab === k ? 'secondary' : 'outline'} class="ml-1 px-1.5">
							{n}
						</Badge>
					{/if}
				</Tabs.Trigger>
			{/each}
		</Tabs.List>
	</Tabs.Root>

	<!-- list -->
	<main
		role="list"
		class="mx-2 mt-0 flex-1 overflow-y-auto rounded-b-lg border border-border bg-card p-2"
		ondragover={onMainDragOver}
		ondrop={onDrop}
	>
		{#if rows.length === 0}
			<p class="p-3 text-center text-muted-foreground">
				{activeCat === null
					? 'Belum ada tugas'
					: curCat
						? `Belum ada tugas di "${curCat.name}"`
						: 'Belum ada kategori'}
			</p>
		{/if}

		{#each view as t, i (t.id)}
			{#if overIdx === i}
				<div class="mx-1 mb-1 h-0.5 rounded-full bg-primary"></div>
			{/if}
			<div
				role="listitem"
				draggable={editId !== t.id && tab !== 'deleted'}
				ondragstart={(e) => onDragStart(e, t)}
				ondragover={(e) => onRowDragOver(e, i)}
				ondrop={onDrop}
				ondragend={onDragEnd}
				class={cn(
					'mb-1.5 flex cursor-grab items-center gap-1.5 rounded-lg border bg-card px-2 py-1.5 transition-shadow active:cursor-grabbing',
					dragId === t.id && 'opacity-35'
				)}
			>
				<GripVertical class="size-3.5 shrink-0 text-muted-foreground/50" />

				{#if editId === t.id}
					<!-- svelte-ignore a11y_autofocus -->
					<Input
						bind:value={editVal}
						autofocus
						onkeydown={onEditKey}
						onblur={() => finishEdit(true)}
						class="h-7 flex-1 rounded-md border-primary px-1.5 py-0.5 text-xs ring-3 ring-primary/15"
					/>
				{:else}
					<div class="flex min-w-0 flex-1 flex-col gap-1">
						{#if t.deleted}
							<div>
								<Badge
									variant="outline"
									class={cn('max-w-32 truncate text-[10px] font-semibold', badgeCls(t.category_id))}
								>
									{byId(t.category_id)?.name ?? ''}
								</Badge>
							</div>
						{:else}
							<div>
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<button {...props} type="button" onclick={() => (moveTask = t)} class="max-w-32 cursor-pointer">
												<Badge
													variant="outline"
													class={cn('max-w-32 truncate text-[10px] font-semibold hover:opacity-75', badgeCls(t.category_id))}
												>
													{byId(t.category_id)?.name ?? 'Tanpa kategori'}
												</Badge>
											</button>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>Ganti kategori</Tooltip.Content>
								</Tooltip.Root>
							</div>
						{/if}
<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<span
									{...props}
									class={cn(
										'min-w-0 cursor-default text-left [overflow-wrap:anywhere]',
										t.done && 'text-muted-foreground line-through',
										t.deleted && 'text-muted-foreground'
									)}
									ondblclick={() => startEdit(t)}
								>
									{t.text}
								</span>
							{/snippet}
						</Tooltip.Trigger>
						{#if !t.deleted && editId !== t.id}
							<Tooltip.Content>Klik 2x untuk edit</Tooltip.Content>
						{/if}
					</Tooltip.Root>
					</div>

					{#if t.deleted}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={() => restore(t)}
										class="border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100/70 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-400 dark:hover:bg-emerald-900/60"
									>
										<RotateCcw />
										Pulihkan
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Pulihkan</Tooltip.Content>
						</Tooltip.Root>
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button {...props} variant="destructive" size="xs" onclick={() => hardDelete(t)}>
										<X />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Hapus permanen</Tooltip.Content>
						</Tooltip.Root>
					{:else if t.done}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button {...props} variant="ghost" size="xs" onclick={() => toggleDone(t)}>
										<Undo2 />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Kembalikan ke Task</Tooltip.Content>
						</Tooltip.Root>
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button {...props} variant="destructive" size="xs" onclick={() => softDelete(t)}>
										<Trash2 />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Hapus</Tooltip.Content>
						</Tooltip.Root>
					{:else}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={() => toggleDone(t)}
										class="border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100/70 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-400 dark:hover:bg-emerald-900/60"
									>
										<Check />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Tandai selesai</Tooltip.Content>
						</Tooltip.Root>
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button {...props} variant="destructive" size="xs" onclick={() => softDelete(t)}>
										<Trash2 />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Hapus</Tooltip.Content>
						</Tooltip.Root>
					{/if}
				{/if}
			</div>
			{#if overIdx === view.length && i === view.length - 1}
				<div class="mx-1 mt-1.5 h-0.5 rounded-full bg-primary"></div>
			{/if}
		{/each}
	</main>

	<!-- footer -->
	<footer class="flex items-center gap-1.5 p-2">
		<div
			class="flex h-8 min-w-0 flex-1 items-center gap-1 rounded-4xl border border-input bg-input/30 pr-1 pl-3 focus-within:border-ring"
		>
			<!-- svelte-ignore a11y_autofocus -->
			<Input
				bind:value={newText}
				placeholder="Tulis tugas, Enter untuk kirim"
				autocomplete="off"
				class="h-8 min-w-0 flex-1 border-0 bg-transparent px-0 shadow-none focus-visible:ring-0 dark:bg-transparent"
				onkeydown={(e) => e.key === 'Enter' && add()}
			/>
			<Select.Root
				type="single"
				value={newCat === null ? '' : String(newCat)}
				onValueChange={(v) => {
					if (v) newCat = Number(v);
				}}
			>
				<Select.Trigger
					size="sm"
					disabled={cats.length === 0}
					class="h-6 w-fit max-w-28 shrink-0 rounded-full border-0 bg-transparent px-2 text-xs shadow-none focus-visible:ring-0"
				>
					<Select.Value placeholder="Kategori">
						{#snippet children()}
							{newCat === null ? 'Kategori' : (byId(newCat)?.name ?? 'Kategori')}
						{/snippet}
					</Select.Value>
				</Select.Trigger>
				<Select.Content>
					{#each cats as c (c.id)}
						<Select.Item value={String(c.id)} label={c.name} />
					{/each}
				</Select.Content>
			</Select.Root>
			<Button
				variant="ghost"
				size="icon-xs"
				title="Tambah kategori baru"
				onclick={() => openCatDialog('add')}
				class="shrink-0 text-muted-foreground hover:text-primary"
			>
				<Plus />
			</Button>
		</div>
		<Button onclick={add} size="sm" class="h-8 shrink-0 px-3.5 text-xs">Kirim</Button>
	</footer>
</div>
{/if}

<Dialog.Root open={exportDialogOpen} onOpenChange={(open) => (exportDialogOpen = open)}>
	<Dialog.Content class="max-w-sm gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Export CSV</Dialog.Title>
			<Dialog.Description class="text-xs">
				Pilih folder dan nama file. Ekstensi .csv akan ditambahkan otomatis.
			</Dialog.Description>
		</Dialog.Header>
		<div class="flex gap-1.5">
			<Input value={exportDirectory} readonly placeholder="Folder tujuan" class="h-8 min-w-0 flex-1 text-xs" />
			<Button variant="outline" size="sm" onclick={chooseExportDirectory}>Pilih folder</Button>
		</div>
		<div class="flex items-center gap-1.5">
			<Input
				bind:value={exportFileName}
				placeholder="Nama file"
				class="h-8 min-w-0 flex-1 text-xs"
				onkeydown={(e) => e.key === 'Enter' && exportCsv()}
			/>
			<span class="text-xs text-muted-foreground">.csv</span>
		</div>
		{#if fileActionError}
			<p class="text-xs text-destructive">{fileActionError}</p>
		{/if}
		<Dialog.Footer class="gap-1.5">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={exportCsv}><Download /> Simpan</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root open={importDialogOpen} onOpenChange={(open) => !importBusy && (importDialogOpen = open)}>
	<Dialog.Content class="max-w-sm gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Import CSV</Dialog.Title>
			<Dialog.Description class="text-xs">
				Gunakan template TaskMini. Kolom todo, project, kategori, done, dan deleted wajib diisi;
				project/kategori akan dibuat otomatis jika belum ada.
			</Dialog.Description>
		</Dialog.Header>
		{#if importBusy}
			<div class="space-y-2">
				<div class="flex justify-between text-xs text-muted-foreground">
					<span>Memasukkan todo...</span>
					<span>{importProcessed} / {importTotal || '...'}</span>
				</div>
				<progress
					class="h-2 w-full overflow-hidden rounded-full accent-primary"
					value={importProcessed}
					max={Math.max(importTotal, 1)}
				></progress>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				<Button onclick={importCsv}><Upload /> Pilih file CSV</Button>
				<Button variant="outline" onclick={saveCsvTemplate}>Simpan template CSV</Button>
			</div>
		{/if}
		{#if fileActionError}
			<p class="text-xs text-destructive">{fileActionError}</p>
		{/if}
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Tutup</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- project picker -->
<Dialog.Root open={projectPickerOpen} onOpenChange={(open) => (projectPickerOpen = open)}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Pilih project</Dialog.Title>
			<Dialog.Description class="text-xs">Beralih ke daftar tugas project lain.</Dialog.Description>
		</Dialog.Header>
		<div class="flex max-h-72 flex-col gap-1.5 overflow-y-auto">
			{#each projects as project (project.id)}
				<Button
					variant={activeProject === project.id ? 'secondary' : 'outline'}
					size="sm"
					class="justify-start gap-2"
					onclick={() => pickProject(project.id)}
				>
					<FolderOpen class="size-4 shrink-0" />
					<span class="truncate">{project.name}</span>
				</Button>
			{/each}
			{#if projects.length === 0}
				<p class="py-3 text-center text-xs text-muted-foreground">Belum ada project.</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Tutup</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- dialog tambah / edit project -->
<Dialog.Root open={projectDialog !== null} onOpenChange={(o) => { if (!o) projectDialog = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>{projectDialog === 'add' ? 'Tambah project' : 'Edit project'}</Dialog.Title>
			<Dialog.Description class="text-xs">
				{projectDialog === 'add' ? 'Project baru akan dibuat dengan kategori Umum.' : 'Perbarui nama dan deskripsi project.'}
			</Dialog.Description>
		</Dialog.Header>
		<!-- svelte-ignore a11y_autofocus -->
		<Input
			bind:value={projectName}
			autofocus
			placeholder="Nama project"
			class="h-8 text-xs"
			onkeydown={(e) => e.key === 'Enter' && submitProject()}
		/>
		<textarea
			bind:value={projectDescription}
			placeholder="Deskripsi singkat (opsional)"
			rows="3"
			class="w-full resize-none rounded-md border border-input bg-transparent px-2.5 py-2 text-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30"
		></textarea>
		{#if projectErr}
			<p class="text-xs text-destructive">{projectErr}</p>
		{/if}
		<Dialog.Footer class="gap-1.5">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={submitProject}>Simpan</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- dialog tambah / ganti nama kategori -->
<Dialog.Root open={catDialog !== null} onOpenChange={(o) => { if (!o) catDialog = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>{catDialog === 'add' ? 'Tambah kategori' : 'Ganti nama kategori'}</Dialog.Title>
			<Dialog.Description class="text-xs">
				{catDialog === 'add' ? 'Kategori baru langsung dipakai.' : `Nama sekarang: ${curCat?.name ?? ''}`}
			</Dialog.Description>
		</Dialog.Header>
		<!-- svelte-ignore a11y_autofocus -->
		<Input
			bind:value={catName}
			autofocus
			placeholder="Nama kategori"
			class="h-8 text-xs"
			onkeydown={(e) => e.key === 'Enter' && submitCat()}
		/>
		<div class="flex flex-wrap gap-1.5">
			{#each PALETTE as c}
				<button
					type="button"
					title={c}
					onclick={() => (catColor = c)}
					class={cn(
						'size-5 cursor-pointer rounded-full border-2 transition-transform',
						solidCls(c),
						catColor === c ? 'scale-110 border-foreground' : 'border-transparent'
					)}
				></button>
			{/each}
		</div>
		{#if catErr}
			<p class="text-xs text-destructive">{catErr}</p>
		{/if}
		<Dialog.Footer class="gap-1.5">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={submitCat}>Simpan</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- modal ganti kategori tugas -->
<Dialog.Root open={moveTask !== null} onOpenChange={(o) => { if (!o) moveTask = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Ganti kategori</Dialog.Title>
			<Dialog.Description class="text-xs [overflow-wrap:anywhere]">{moveTask?.text ?? ''}</Dialog.Description>
		</Dialog.Header>
		<div class="flex flex-col gap-1.5">
			{#each cats as c (c.id)}
				<Button
					variant={moveTask?.category_id === c.id ? 'secondary' : 'outline'}
					size="sm"
					class="gap-2 justify-start"
					onclick={async () => {
						const t = moveTask;
						moveTask = null;
						if (t) await moveCat(t, c.id);
					}}
				>
					<span class={cn('size-2.5 shrink-0 rounded-full', solidCls(c.color))}></span>
					{c.name}
				</Button>
			{/each}
			{#if cats.length === 0}
				<p class="text-xs text-muted-foreground">Belum ada kategori.</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Batal</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- konfirmasi hapus -->
<AlertDialog.Root open={confirmState !== null} onOpenChange={(o) => { if (!o) confirmState = null; }}>
	<AlertDialog.Content class="max-w-xs gap-4 p-4">
		<AlertDialog.Header class="gap-1">
			<AlertDialog.Title>Konfirmasi</AlertDialog.Title>
			<AlertDialog.Description class="text-xs">{confirmState?.msg ?? ''}</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="gap-1.5">
			<AlertDialog.Cancel variant="outline" size="sm">Batal</AlertDialog.Cancel>
			<AlertDialog.Action
				size="sm"
				onclick={async () => {
					const run = confirmState?.run;
					confirmState = null;
					if (run) await run();
				}}
			>
				OK
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<Toaster closeButton theme={dark ? 'dark' : 'light'} position="top-center" />
