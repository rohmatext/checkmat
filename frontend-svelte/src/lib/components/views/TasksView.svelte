<script lang="ts">
	import {
		ArrowLeft,
		FolderOpen,
		FolderInput,
		Tags,
		Pencil,
		Trash,
		Lock,
		Sun,
		Moon,
		Upload,
		Download,
		ArrowUpDown,
		ArrowUp,
		ArrowDown,
		GripVertical,
		Check,
		RotateCcw,
		Undo2,
		Plus,
		X,
		Ellipsis,
		EllipsisVertical,
		ListChecks,
		Info,
		Copy,
		Eye,
		ChevronDown,
		Clock,
		Circle,
		CircleCheck,
		CircleDashed,
		CircleDashedCheck,
		CircleDot,
		ArrowLeftRight,
		ShieldCheck
	} from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		InputGroup,
		InputGroupAddon,
		InputGroupButton,
		InputGroupInput,
		InputGroupTextarea
	} from '#lib/components/ui/input-group/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Tabs from '#lib/components/ui/tabs/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';
	import CategorySelect from '#lib/components/CategorySelect.svelte';
	import TaskDetailDialog from '#lib/components/dialogs/TaskDetailDialog.svelte';
	import ProjectDetailDialog from '#lib/components/dialogs/ProjectDetailDialog.svelte';
	import CopyableText from '#lib/components/CopyableText.svelte';
	import BatchMoveCategoryDialog from '#lib/components/dialogs/BatchMoveCategoryDialog.svelte';
	import TaskCompareDialog from '#lib/components/dialogs/TaskCompareDialog.svelte';
	import BatchDuplicateToCategoriesDialog from '#lib/components/dialogs/BatchDuplicateToCategoriesDialog.svelte';
	import CustomContextMenu from '#lib/components/CustomContextMenu.svelte';
	import { formatFullDate, formatTaskDate, getBadgeCls, getSolidCls, getTaskEventTooltip, type Category, type Project, type ProjectStatus, type SortDir, type SortField, type Tab, type Task, type TaskStatus } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		tasks = [],
		cats = [],
		activeProjectItem = undefined,
		activeCats = $bindable([]),
		newCat = $bindable(null),
		tab = $bindable('active'),
		sortField = $bindable('cat_pos'),
		sortDir = $bindable('asc'),
		dark = false,
		onBackToProjects,
		onOpenProjectPicker,
		onOpenCategoriesView,
		onOpenRenameCategory,
		onDeleteCategory,
		onOpenAddCategory,
		onLockApp,
		onToggleTheme,
		onOpenImportDialog,
		onOpenExportDialog,
		onAddTask,
		onUpdateTask,
		onDuplicateTask,
		onDuplicateToCategories,
		onToggleDone,
		onSetTaskStatus,
		onSetTaskStatusBatch,
		onSetDoneBatch,
		onSoftDelete,
		onSoftDeleteBatch,
		onRestore,
		onRestoreBatch,
		onHardDelete,
		onHardDeleteBatch,
		onDuplicateBatch,
		onSetCategoryBatch,
		onMoveTaskCategory,
		onReorderTasks,
		onOpenAbout,
		onOpenSecuritySettings,
		onOpenEditProject,
		onSetProjectStatus,
		onRefreshTasks
	}: {
		tasks: Task[];
		cats: Category[];
		activeProjectItem?: Project;
		activeCats: number[];
		newCat: number | null;
		tab: Tab;
		sortField: SortField;
		sortDir: SortDir;
		dark: boolean;
		onBackToProjects: () => void;
		onOpenProjectPicker: () => void;
		onOpenCategoriesView: () => void;
		onOpenRenameCategory: () => void;
		onDeleteCategory: () => void;
		onOpenAddCategory: () => void;
		onLockApp: () => void;
		onToggleTheme: () => void;
		onOpenImportDialog: () => void;
		onOpenExportDialog: () => void;
		onAddTask: (text: string, catId: number, description?: string) => Promise<void>;
		onUpdateTask: (id: number, text: string, catId: number, description?: string) => Promise<void>;
		onDuplicateTask?: (t: Task) => void;
		onDuplicateTaskToCategories?: (t: Task) => void;
		onToggleDone: (t: Task) => void;
		onSetTaskStatus?: (id: number, status: TaskStatus) => Promise<void>;
		onSetTaskStatusBatch?: (ids: number[], status: TaskStatus, onSuccess?: () => void) => Promise<void>;
		onSetDoneBatch?: (ids: number[], done: boolean, onSuccess?: () => void) => void;
		onSoftDelete: (t: Task) => void;
		onSoftDeleteBatch?: (ids: number[], onSuccess?: () => void) => void;
		onRestore: (t: Task) => Promise<void>;
		onRestoreBatch?: (ids: number[], onSuccess?: () => void) => Promise<void>;
		onHardDelete: (t: Task) => void;
		onHardDeleteBatch?: (ids: number[], onSuccess?: () => void) => void;
		onDuplicateBatch?: (ids: number[], onSuccess?: () => void) => void;
		onSetCategoryBatch?: (ids: number[], categoryId: number, onSuccess?: () => void) => void;
		onMoveTaskCategory: (t: Task) => void;
		onReorderTasks: (ids: number[]) => Promise<void>;
		onOpenAbout?: () => void;
		onOpenSecuritySettings?: () => void;
		onOpenEditProject?: () => void;
		onSetProjectStatus?: (id: number, status: ProjectStatus) => void;
		onRefreshTasks?: () => Promise<void> | void;
	} = $props();

	let newText = $state('');
	let showDesc = $state(false);
	let newDesc = $state('');
	let newTextInput: HTMLTextAreaElement | null = $state(null);
	let descInput: HTMLTextAreaElement | null = $state(null);
	let editId = $state<number | null>(null);
	let editVal = $state('');
	let editDesc = $state('');
	let editCat = $state<number | null>(null);
	let showEditDesc = $state(false);
	let editInput: HTMLTextAreaElement | null = $state(null);
	let editDescInput: HTMLTextAreaElement | null = $state(null);
	let confirmDiscard = $state(false);
	let skipEditClose = $state(false);

	let expandedIds = $state<number[]>([]);
	let detailTask = $state<Task | null>(null);
	let showSpaceDetail = $state(false);
	let truncState = $state<Record<number, { t: boolean; d: boolean }>>({});
	let contextMenuState = $state<{
		type: 'task' | 'background';
		task?: Task;
		x: number;
		y: number;
	} | null>(null);

	function handleTaskContextMenu(e: MouseEvent, t: Task) {
		if (selectMode) return;
		e.preventDefault();
		e.stopPropagation();
		contextMenuState = {
			type: 'task',
			task: t,
			x: e.clientX,
			y: e.clientY
		};
	}

	function handleBgContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		if (target?.closest('input, textarea, button, [role="button"], [role="menuitem"]')) {
			return;
		}
		e.preventDefault();
		e.stopPropagation();
		contextMenuState = {
			type: 'background',
			x: e.clientX,
			y: e.clientY
		};
	}

	// Deteksi teks yang terpotong ellipsis (judul / deskripsi) supaya expand
	// hanya offers saat memang ada yang terpotong.
	function trackTrunc(node: HTMLElement, params: { id: number; kind: 't' | 'd' }) {
		const update = () => {
			const over = node.scrollWidth > node.clientWidth + 1;
			const cur = truncState[params.id] ?? { t: false, d: false };
			if (cur[params.kind] !== over) {
				truncState = { ...truncState, [params.id]: { ...cur, [params.kind]: over } };
			}
		};
		update();
		const ro = new ResizeObserver(update);
		ro.observe(node);
		return {
			destroy() {
				ro.disconnect();
			}
		};
	}

	function toggleExpand(id: number) {
		if (expandedIds.includes(id)) {
			expandedIds = expandedIds.filter((x) => x !== id);
		} else {
			expandedIds = [...expandedIds, id];
		}
	}

	function splitDesc(s: string) {
		const m = s.match(/(?:^|\s)(?:desc|d:|note:)\s+(.*)$/i);
		if (m) {
			const desc = m[1].trim();
			const text = s.slice(0, m.index).trim();
			return { text, desc };
		}
		return { text: s.trim(), desc: '' };
	}

	function insertCopyTag(kind: 'new' | 'edit') {
		if (kind === 'new') {
			const el = newTextInput;
			if (el) {
				const start = el.selectionStart ?? newText.length;
				const end = el.selectionEnd ?? newText.length;
				const sel = newText.slice(start, end);
				newText = newText.slice(0, start) + `[copy]${sel}[/copy]` + newText.slice(end);
				setTimeout(() => {
					el.focus();
					const cursor = start + 6 + sel.length;
					el.setSelectionRange(cursor, cursor);
					resizeTextarea(el);
				}, 0);
			} else {
				newText += '[copy][/copy]';
			}
		} else {
			const el = editInput;
			if (el) {
				const start = el.selectionStart ?? editVal.length;
				const end = el.selectionEnd ?? editVal.length;
				const sel = editVal.slice(start, end);
				editVal = editVal.slice(0, start) + `[copy]${sel}[/copy]` + editVal.slice(end);
				setTimeout(() => {
					el.focus();
					const cursor = start + 6 + sel.length;
					el.setSelectionRange(cursor, cursor);
					resizeTextarea(el);
				}, 0);
			} else {
				editVal += '[copy][/copy]';
			}
		}
	}

	$effect(() => {
		if (showDesc) descInput?.focus();
	});

	$effect(() => {
		if (!confirmDiscard && editId !== null) editInput?.focus();
	});
	let dragId = $state<number | null>(null);
	let overIdx = $state<number | null>(null);
	let dragRows = $state<Task[] | null>(null);

	// Batch selection state
	let selectMode = $state(false);
	let selectedIds = $state<number[]>([]);
	let batchCategoryDialogOpen = $state(false);
	let batchDuplicateDialogOpen = $state(false);
	let duplicateToCategoriesTask = $state<Task | null>(null);

	// Compare tasks state
	let compareOpen = $state(false);
	let compareTaskA = $state<Task | null>(null);
	let compareTaskB = $state<Task | null>(null);

	$effect(() => {
		// Reset pilihan saat tab atau kategori aktif berubah
		tab;
		activeCats;
		selectedIds = [];
	});

	const byId = (id: number | null) => cats.find((c) => c.id === id);
	const singleCat = $derived(activeCats.length === 1);
	const curCat = $derived(singleCat ? byId(activeCats[0]) : undefined);
	const isAllCats = $derived(activeCats.length === 0);
	const inActiveCats = (t: Task) => isAllCats || activeCats.includes(t.category_id);

	const inTab = (t: Task, k: Tab) =>
		k === 'active' || k === 'todo'
			? !t.deleted && !t.done && (!t.status || t.status === 'todo')
			: k === 'inprogress'
				? !t.deleted && !t.done && t.status === 'inprogress'
				: k === 'done'
					? !t.deleted && (t.done || t.status === 'done')
					: t.deleted;

	const count = (k: Tab) =>
		tasks.filter((t) => inActiveCats(t) && inTab(t, k)).length;

	function cycleTaskStatus(t: Task) {
		const current = t.done || t.status === 'done' ? 'done' : t.status === 'inprogress' ? 'inprogress' : 'todo';
		if (current === 'todo') {
			if (onSetTaskStatus) {
				onSetTaskStatus(t.id, 'inprogress');
			} else {
				onToggleDone(t);
			}
		} else if (current === 'inprogress') {
			if (onSetTaskStatus) {
				onSetTaskStatus(t.id, 'done');
			} else {
				onToggleDone(t);
			}
		} else {
			if (onSetTaskStatus) {
				onSetTaskStatus(t.id, 'todo');
			} else {
				onToggleDone(t);
			}
		}
	}

	function setStatusDirect(t: Task, st: TaskStatus) {
		if (onSetTaskStatus) {
			onSetTaskStatus(t.id, st);
		} else {
			if (st === 'done' && !t.done) onToggleDone(t);
			else if (st !== 'done' && t.done) onToggleDone(t);
		}
	}

	let rows = $derived.by(() => {
		const filtered = tasks.filter((t) => inActiveCats(t) && inTab(t, tab));
		if (!isAllCats) {
			return filtered;
		}

		const list = [...filtered];
		list.sort((a, b) => {
			let comp = 0;
			if (sortField === 'cat_pos') {
				const cA = byId(a.category_id);
				const cB = byId(b.category_id);
				const posCatA = cA?.pos ?? cats.findIndex((c) => c.id === a.category_id);
				const posCatB = cB?.pos ?? cats.findIndex((c) => c.id === b.category_id);
				comp = posCatA - posCatB;
				if (comp === 0) comp = a.pos - b.pos;
			} else if (sortField === 'pos') {
				comp = a.pos - b.pos;
			} else if (sortField === 'category') {
				const catA = (byId(a.category_id)?.name ?? '').toLowerCase();
				const catB = (byId(b.category_id)?.name ?? '').toLowerCase();
				comp = catA.localeCompare(catB);
				if (comp === 0) comp = a.pos - b.pos;
			} else if (sortField === 'id') {
				comp = a.id - b.id;
			} else if (sortField === 'text') {
				comp = a.text.localeCompare(b.text);
			}

			return sortDir === 'asc' ? comp : -comp;
		});
		return list;
	});

	let view = $derived(dragRows ?? rows);

	function sortFieldLabel(f: SortField): string {
		switch (f) {
			case 'cat_pos':
				return 'Kategori + Todo Posisi';
			case 'pos':
				return 'Todo Posisi';
			case 'category':
				return 'Nama Kategori';
			case 'id':
				return 'Waktu dibuat';
			case 'text':
				return 'Nama todo';
		}
	}

	function handlePickCat(v: string) {
		activeCats = v === 'all' ? [] : [Number(v)];
		if (activeCats.length === 1) newCat = activeCats[0];
		dragRows = null;
		dragId = null;
		overIdx = null;
	}

	function toggleCat(id: number) {
		activeCats = activeCats.includes(id)
			? activeCats.filter((x) => x !== id)
			: [...activeCats, id];
		newCat = id;
		dragRows = null;
		dragId = null;
		overIdx = null;
	}

	function toggleSortDir() {
		sortDir = sortDir === 'asc' ? 'desc' : 'asc';
	}

	const allRowsSelected = $derived(
		rows.length > 0 && rows.every((r) => selectedIds.includes(r.id))
	);

	function toggleSelect(id: number) {
		if (selectedIds.includes(id)) {
			selectedIds = selectedIds.filter((x) => x !== id);
		} else {
			selectedIds = [...selectedIds, id];
		}
	}

	function toggleSelectAll() {
		if (allRowsSelected) {
			selectedIds = [];
		} else {
			selectedIds = rows.map((r) => r.id);
		}
	}

	function exitSelectMode() {
		selectMode = false;
		selectedIds = [];
	}

	function handleBatchDeleteAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (tab === 'deleted') {
			if (onHardDeleteBatch) {
				onHardDeleteBatch(ids, () => {
					selectedIds = [];
				});
			}
		} else {
			if (onSoftDeleteBatch) {
				onSoftDeleteBatch(ids, () => {
					selectedIds = [];
				});
			}
		}
	}

	async function handleBatchRestoreAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onRestoreBatch) {
			await onRestoreBatch(ids, () => {
				selectedIds = [];
			});
		}
	}

	function handleBatchSetDoneAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onSetDoneBatch) {
			onSetDoneBatch(ids, true, () => {
				selectedIds = [];
			});
		}
	}

	function handleBatchSetUndoneAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onSetDoneBatch) {
			onSetDoneBatch(ids, false, () => {
				selectedIds = [];
			});
		}
	}

	function handleBatchDuplicateAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onDuplicateBatch) {
			onDuplicateBatch(ids, () => {
				selectedIds = [];
			});
		}
	}

	function handleBatchSetStatusAction(status: TaskStatus) {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onSetTaskStatusBatch) {
			onSetTaskStatusBatch(ids, status, () => {
				selectedIds = [];
			});
		}
	}

	function handleBatchSetCategoryAction(catId: number) {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onSetCategoryBatch) {
			onSetCategoryBatch(ids, catId, () => {
				selectedIds = [];
			});
		}
	}

	function openCompare(t: Task) {
		compareTaskA = t;
		compareTaskB = null;
		compareOpen = true;
	}

	function handleBatchCompareAction() {
		if (selectedIds.length !== 2) return;
		const tA = tasks.find((t) => t.id === selectedIds[0]);
		const tB = tasks.find((t) => t.id === selectedIds[1]);
		if (!tA || !tB) return;
		compareTaskA = tA;
		compareTaskB = tB;
		compareOpen = true;
	}

	function resizeTextarea(el: HTMLTextAreaElement | null) {
		if (!el) return;
		el.style.height = 'auto';
		el.style.height = `${el.scrollHeight}px`;
	}

	function startEdit(t: Task) {
		if (t.deleted) return;
		editId = t.id;
		editVal = t.text;
		editDesc = t.description ?? '';
		editCat = t.category_id;
		showEditDesc = !!t.description;
		skipEditClose = false;
		setTimeout(() => {
			resizeTextarea(editInput);
			if (showEditDesc) {
				resizeTextarea(editDescInput);
			}
		}, 0);
	}

	async function finishEdit(save: boolean) {
		if (skipEditClose) {
			skipEditClose = false;
			return;
		}
		confirmDiscard = false;
		const id = editId;
		const t = tasks.find((x) => x.id === id);
		editId = null;
		if (!t) return;
		const val = editVal.trim();
		const desc = editDesc.trim();
		if (
			save &&
			val &&
			(val !== t.text ||
				desc !== (t.description ?? '') ||
				editCat !== t.category_id)
		) {
			await onUpdateTask(t.id, val, editCat ?? t.category_id, desc || undefined);
		}
	}

	const onEditKey = (e: KeyboardEvent) => {
		if (e.key === 'Enter') finishEdit(true);
		else if (e.key === 'Escape') finishEdit(false);
	};

	const onEditDescKey = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			finishEdit(false);
		} else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			finishEdit(true);
		}
	};

	async function submitAdd() {
		const raw = newText;
		const { text, desc: parsedDesc } = splitDesc(raw);
		if (!text) return;
		const catId = newCat ?? cats[0]?.id;
		if (catId == null) {
			alert('Belum ada kategori.');
			return;
		}
		const desc = (newDesc.trim() || parsedDesc).trim();
		newText = '';
		newDesc = '';
		showDesc = false;
		tab = 'active';
		setTimeout(() => {
			resizeTextarea(newTextInput);
		}, 0);
		await onAddTask(text, catId, desc || undefined);
	}

	/* ----- drag reorder ----- */
	function shift(idx: number) {
		if (!singleCat) return;
		const cur = dragRows ?? rows;
		const from = cur.findIndex((r) => r.id === dragId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragRows = next;
	}

	function onDragStart(e: DragEvent, t: Task) {
		if (!singleCat) return;
		dragId = t.id;
		dragRows = [...rows];
		overIdx = rows.findIndex((r) => r.id === t.id);
		e.dataTransfer?.setData('text/plain', String(t.id));
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onRowDragOver(e: DragEvent, i: number) {
		if (!singleCat) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const mid = rect.top + rect.height / 2;
		const insertIdx = e.clientY < mid ? i : i + 1;
		if (insertIdx !== overIdx) {
			overIdx = insertIdx;
			shift(insertIdx);
		}
	}

	function onMainDragOver(e: DragEvent) {
		if (!singleCat) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const items = document.querySelectorAll('[role="listitem"]');
		if (items.length === 0) return;
		const last = items[items.length - 1].getBoundingClientRect();
		if (e.clientY > last.bottom && overIdx !== items.length) {
			overIdx = items.length;
			shift(items.length);
		}
	}

	async function onDrop(e: DragEvent) {
		if (!singleCat) return;
		e.preventDefault();
		const final = dragRows;
		dragId = null;
		overIdx = null;
		dragRows = null;
		if (!final) return;
		const currentIds = rows.map((r) => r.id);
		const newIds = final.map((r) => r.id);
		if (newIds.every((id, idx) => id === currentIds[idx])) return;
		await onReorderTasks(newIds);
	}

	function onDragEnd() {
		dragId = null;
		overIdx = null;
		dragRows = null;
	}

	const mainTabs: { k: Tab; label: string }[] = [
		{ k: 'active', label: 'Todo' },
		{ k: 'inprogress', label: 'In Progress' },
		{ k: 'done', label: 'Selesai' }
	];
</script>

<div class="flex h-full flex-col">
	<!-- project title and navigation + app utilities -->
	<div class="flex items-center justify-between gap-1 px-2 pt-2">
		<div class="flex min-w-0 flex-1 items-center gap-1">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" onclick={onBackToProjects}>
							<ArrowLeft />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Kembali ke space</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<button
							{...props}
							type="button"
							class="min-w-0 flex-1 truncate px-2 text-left text-sm font-bold hover:text-primary cursor-pointer"
							onclick={() => (showSpaceDetail = true)}
						>
							{activeProjectItem?.name ?? 'Pilih space'}
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Detail space</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="ghost" size="icon-sm" class="text-muted-foreground hover:text-foreground shrink-0" onclick={onOpenProjectPicker}>
							<FolderOpen />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Pilih space</Tooltip.Content>
			</Tooltip.Root>
		</div>

		<!-- Action aplikasi di baris atas -->
		<div class="flex items-center gap-1 shrink-0">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" onclick={onToggleTheme}>
							{#if dark}<Sun />{:else}<Moon />{/if}
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>{dark ? 'Mode terang' : 'Mode gelap'}</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" onclick={onLockApp}>
							<Lock />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Kunci aplikasi</Tooltip.Content>
			</Tooltip.Root>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm">
							<EllipsisVertical />
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="min-w-40 w-40 rounded-lg p-1 shadow-lg border border-border">
					<DropdownMenu.Item onclick={onOpenImportDialog} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
						<Upload class="size-3.5" /> Import CSV
					</DropdownMenu.Item>
					<DropdownMenu.Item onclick={onOpenExportDialog} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
						<Download class="size-3.5" /> Export CSV
					</DropdownMenu.Item>
					<DropdownMenu.Separator />
					<DropdownMenu.Item onclick={onOpenCategoriesView} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
						<Tags class="size-3.5" /> Kelola Kategori
					</DropdownMenu.Item>
					{#if onOpenSecuritySettings}
						<DropdownMenu.Item onclick={onOpenSecuritySettings} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
							<ShieldCheck class="size-3.5 text-primary" /> Pengaturan Kunci
						</DropdownMenu.Item>
					{/if}
					{#if onOpenAbout}
						<DropdownMenu.Separator />
						<DropdownMenu.Item onclick={onOpenAbout} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
							<Info class="size-3.5" /> Tentang Aplikasi
						</DropdownMenu.Item>
					{/if}
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
	</div>

	<!-- filter kategori -->
	<div class="flex flex-col gap-1 px-2 pt-2">
		<div class="flex items-center gap-1.5">
			<div class="min-w-0 flex-1">
				<CategorySelect
					{cats}
					multiple={true}
					values={activeCats}
					allowAll={true}
					allLabel="Semua kategori"
					{tasks}
					onToggle={toggleCat}
					onClear={() => handlePickCat('all')}
					{onOpenAddCategory}
				/>
			</div>

		<div class="flex items-center gap-1 shrink-0">
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" onclick={onOpenCategoriesView}>
							<Tags />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Kelola kategori</Tooltip.Content>
			</Tooltip.Root>
			{#if activeCats.length > 0}
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="outline"
								size="icon-sm"
								onclick={() => handlePickCat('all')}
							>
								<X />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Bersihkan filter</Tooltip.Content>
				</Tooltip.Root>
			{/if}
			{#if singleCat}
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button {...props} variant="outline" size="icon-sm">
								<Ellipsis />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end" class="min-w-36 w-36 rounded-lg p-1 shadow-lg border border-border">
						<DropdownMenu.Item onclick={onOpenRenameCategory} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
							<Pencil class="size-3.5" /> Edit Kategori
						</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item
							onclick={onDeleteCategory}
							class="py-1.5 px-2 text-xs rounded-md gap-2 text-destructive focus:bg-destructive/10 focus:text-destructive cursor-pointer"
						>
							<Trash class="size-3.5" /> Hapus Kategori
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			{/if}
		</div>
		</div>

		{#if activeCats.length > 0}
			<div class="flex flex-wrap items-center gap-1">
				{#each cats.filter((c) => activeCats.includes(c.id)) as c (c.id)}
					<button
						type="button"
						onclick={() => toggleCat(c.id)}
						title="Hapus filter {c.name}"
						class="group flex max-w-40 items-center gap-1 rounded-full border border-border bg-muted/50 py-0.5 pl-1.5 pr-1 text-[10px] font-medium text-foreground transition-colors hover:bg-accent cursor-pointer"
					>
						<span class={cn('size-1.5 shrink-0 rounded-full', getSolidCls(c.color))}></span>
						<span class="truncate">{c.name}</span>
						<X class="size-3 shrink-0 opacity-60 group-hover:opacity-100" />
					</button>
				{/each}
				{#if activeCats.length > 1}
					<button
						type="button"
						onclick={() => handlePickCat('all')}
						class="rounded-full px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground transition-colors hover:text-foreground cursor-pointer"
					>
						Hapus semua
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<!-- tabs -->
	<Tabs.Root
		value={tab}
		onValueChange={(v) => {
			if (v) {
				tab = v as Tab;
				selectedIds = [];
			}
		}}
		class="gap-0 px-2 pt-2"
	>
		<Tabs.List class="flex w-full items-stretch gap-1 rounded-t-lg rounded-b-none bg-transparent p-0">
			{#each mainTabs as { k, label } (k)}
				{@const n = count(k)}
				<Tabs.Trigger
					value={k}
					class="flex-1 cursor-pointer rounded-t-lg border border-b-0 border-border px-1.5 py-1 text-xs font-semibold text-muted-foreground data-[state=active]:bg-card data-[state=active]:text-primary data-[state=active]:shadow-[inset_0_-1px_0_var(--color-primary)] flex items-center justify-center gap-1"
				>
					<span>{label}</span>
					{#if n > 0}
						<Badge variant={(tab === k || (k === 'active' && tab === 'todo')) ? 'secondary' : 'outline'} class="px-1 py-0 text-[10px] leading-tight font-medium">
							{n}
						</Badge>
					{/if}
				</Tabs.Trigger>
			{/each}
			{@const trashN = count('deleted')}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Tabs.Trigger
							{...props}
							value="deleted"
							class="cursor-pointer shrink-0 rounded-t-lg border border-b-0 border-border px-2.5 py-1 text-xs font-semibold text-muted-foreground data-[state=active]:bg-card data-[state=active]:text-destructive data-[state=active]:shadow-[inset_0_-1px_0_var(--color-destructive)] flex items-center justify-center gap-1"
						>
							<Trash class="size-3.5" />
							{#if trashN > 0}
								<Badge variant={tab === 'deleted' ? 'secondary' : 'outline'} class="px-1 py-0 text-[10px] leading-tight font-medium text-destructive border-destructive/30">
									{trashN}
								</Badge>
							{/if}
						</Tabs.Trigger>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Sampah / Dihapus ({trashN})</Tooltip.Content>
			</Tooltip.Root>
		</Tabs.List>
	</Tabs.Root>

	<!-- list -->
	<main
		role="list"
		class="mx-2 mt-0 flex-1 overflow-y-auto rounded-b-lg border border-border bg-card p-2"
		ondragover={onMainDragOver}
		ondrop={onDrop}
		oncontextmenu={handleBgContextMenu}
	>
		<!-- Batch Toolbar atau Normal Baris Urutan / Tombol Pilih -->
		{#if selectMode}
			<div class="mb-2 flex items-center justify-between gap-1.5 rounded-lg border border-primary/30 bg-primary/10 px-2 py-1.5 text-xs animate-in fade-in-50">
				<div class="flex items-center gap-2 min-w-0">
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={cn(
										'size-4 shrink-0 rounded border transition-colors flex items-center justify-center cursor-pointer',
										allRowsSelected
											? 'border-primary bg-primary text-primary-foreground'
											: 'border-muted-foreground/40 hover:border-foreground bg-background'
									)}
									onclick={toggleSelectAll}
								>
									{#if allRowsSelected}
										<Check class="size-3 stroke-[3]" />
									{/if}
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>{allRowsSelected ? 'Batal tandai semua' : 'Tandai semua'}</Tooltip.Content>
					</Tooltip.Root>
					<span class="text-[11px] font-semibold truncate">
						{selectedIds.length} ditandai
					</span>
				</div>

				<div class="flex items-center gap-1 shrink-0">
					{#if selectedIds.length > 0}
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="default"
										size="xs"
										class="h-6 px-2.5 text-[11px] gap-1.5 font-medium cursor-pointer shadow-sm"
									>
										<span>Pilih Aksi ({selectedIds.length})</span>
										<ChevronDown class="size-3 opacity-80" />
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>
							<DropdownMenu.Content align="end" class="min-w-44 p-1 rounded-lg border border-border shadow-lg">
								{#if tab === 'deleted'}
									<DropdownMenu.Item
										onclick={handleBatchRestoreAction}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<RotateCcw class="size-3.5 text-muted-foreground" />
										<span>Pulihkan Todo</span>
									</DropdownMenu.Item>
									<DropdownMenu.Separator />
									<DropdownMenu.Item
										onclick={handleBatchDeleteAction}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-destructive focus:text-destructive"
									>
										<Trash class="size-3.5" />
										<span>Hapus Permanen</span>
									</DropdownMenu.Item>
								{:else}
									{#if onSetTaskStatusBatch}
										<div class="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
											Status
										</div>
										{#if tab !== 'active' && tab !== 'todo'}
											<DropdownMenu.Item
												onclick={() => handleBatchSetStatusAction('todo')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Circle class="size-3.5 text-blue-500" />
												<span>Ubah ke Todo</span>
											</DropdownMenu.Item>
										{/if}
										{#if tab !== 'inprogress'}
											<DropdownMenu.Item
												onclick={() => handleBatchSetStatusAction('inprogress')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Clock class="size-3.5 text-amber-500" />
												<span>Ubah ke In Progress</span>
											</DropdownMenu.Item>
										{/if}
										{#if tab !== 'done'}
											<DropdownMenu.Item
												onclick={() => handleBatchSetStatusAction('done')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Check class="size-3.5 text-emerald-500" />
												<span>Tandai Selesai</span>
											</DropdownMenu.Item>
										{/if}
										<DropdownMenu.Separator />
									{/if}
									<div class="px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
										Kelola
									</div>
									<DropdownMenu.Item
										onclick={() => (batchCategoryDialogOpen = true)}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<FolderInput class="size-3.5 text-muted-foreground" />
										<span>Ganti Kategori...</span>
									</DropdownMenu.Item>
									<DropdownMenu.Item
										onclick={handleBatchDuplicateAction}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<Copy class="size-3.5 text-muted-foreground" />
										<span>Duplikat ({selectedIds.length})</span>
									</DropdownMenu.Item>
									{#if selectedIds.length === 2}
										<DropdownMenu.Item
											onclick={handleBatchCompareAction}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-primary focus:text-primary font-medium"
										>
											<ArrowLeftRight class="size-3.5" />
											<span>Bandingkan 2 Todo</span>
										</DropdownMenu.Item>
									{/if}
									<DropdownMenu.Separator />
									<DropdownMenu.Item
										onclick={handleBatchDeleteAction}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-destructive focus:text-destructive"
									>
										<Trash class="size-3.5" />
										<span>Hapus ({selectedIds.length})</span>
									</DropdownMenu.Item>
								{/if}
							</DropdownMenu.Content>
						</DropdownMenu.Root>
					{:else}
						<Button
							variant="outline"
							size="xs"
							disabled
							class="h-6 px-2 text-[11px] gap-1 opacity-50"
						>
							Pilih Aksi
						</Button>
					{/if}
					<Button
						variant="ghost"
						size="xs"
						class="h-6 px-2 text-[11px] text-muted-foreground hover:text-foreground cursor-pointer"
						onclick={exitSelectMode}
					>
						Batal
					</Button>
				</div>
			</div>
		{:else if rows.length > 0}
			<div class="mb-2 flex items-center justify-between gap-1">
				{#if isAllCats}
					<div class="flex items-center gap-1 rounded-lg bg-muted/40 p-0.5 text-xs">
						<span class="flex items-center gap-1 pl-1.5 pr-1 text-[11px] font-medium text-muted-foreground select-none">
							<ArrowUpDown class="size-3 text-muted-foreground" />
							Urutan:
						</span>
						<Select.Root
							type="single"
							value={sortField}
							onValueChange={(v) => v && (sortField = v as SortField)}
						>
							<Select.Trigger
								size="sm"
								class="!h-6 w-fit gap-1 rounded-md border-0 bg-background/80 shadow-xs !px-1.5 !text-[11px] font-medium hover:bg-background [&_svg]:!size-3 cursor-pointer"
							>
								<Select.Value>
									{#snippet children()}
										{sortFieldLabel(sortField)}
									{/snippet}
								</Select.Value>
							</Select.Trigger>
							<Select.Content class="min-w-44 rounded-lg p-1 shadow-lg border border-border">
								<Select.Item class="py-1 pl-2 pr-6 text-xs rounded-md gap-1.5 cursor-pointer [&_svg]:size-3.5" value="cat_pos" label="Kategori + Todo Posisi" />
								<Select.Item class="py-1 pl-2 pr-6 text-xs rounded-md gap-1.5 cursor-pointer [&_svg]:size-3.5" value="pos" label="Todo Posisi" />
								<Select.Item class="py-1 pl-2 pr-6 text-xs rounded-md gap-1.5 cursor-pointer [&_svg]:size-3.5" value="category" label="Nama Kategori" />
								<Select.Item class="py-1 pl-2 pr-6 text-xs rounded-md gap-1.5 cursor-pointer [&_svg]:size-3.5" value="id" label="Waktu dibuat" />
								<Select.Item class="py-1 pl-2 pr-6 text-xs rounded-md gap-1.5 cursor-pointer [&_svg]:size-3.5" value="text" label="Nama todo" />
							</Select.Content>
						</Select.Root>
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon-xs"
										class="!size-6 rounded-md hover:bg-background/80 text-muted-foreground hover:text-foreground"
										onclick={toggleSortDir}
									>
										{#if sortDir === 'asc'}
											<ArrowUp class="size-3" />
										{:else}
											<ArrowDown class="size-3" />
										{/if}
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>{sortDir === 'asc' ? 'Urutan Naik (A-Z / Lama ke Baru)' : 'Urutan Turun (Z-A / Baru ke Lama)'}</Tooltip.Content>
						</Tooltip.Root>
					</div>
				{:else}
					<div class="text-[11px] text-muted-foreground px-1">
						{rows.length} todo
					</div>
				{/if}

				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="outline"
								size="xs"
								class="h-6 gap-1 px-2 text-[11px] text-muted-foreground hover:text-foreground"
								onclick={() => (selectMode = true)}
							>
								<ListChecks class="size-3.5 text-primary" />
								<span>Tandai</span>
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Tandai beberapa todo sekaligus</Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}

		{#if rows.length === 0}
			<p class="p-3 text-center text-muted-foreground">
				{activeCats.length === 0
					? 'Belum ada todo'
					: curCat
						? `Belum ada todo di "${curCat.name}"`
						: 'Belum ada kategori'}
			</p>
		{/if}

		{#each view as t, i (t.id)}
			{@const isSelected = selectedIds.includes(t.id)}
			{@const isExpanded = expandedIds.includes(t.id)}
			{#if singleCat && overIdx === i}
				<div class="mx-1 mb-1 h-0.5 rounded-full bg-primary"></div>
			{/if}
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div
				role="listitem"
				draggable={!selectMode && editId !== t.id && tab !== 'deleted' && singleCat}
				ondragstart={(e) => onDragStart(e, t)}
				ondragover={(e) => onRowDragOver(e, i)}
				ondrop={onDrop}
				ondragend={onDragEnd}
				onclick={() => {
					if (selectMode) toggleSelect(t.id);
				}}
				oncontextmenu={(e) => handleTaskContextMenu(e, t)}
				class={cn(
					'mb-1.5 flex items-start gap-1.5 rounded-lg border bg-card px-2 py-1.5 transition-all select-none',
					singleCat && tab !== 'deleted' && !selectMode && editId !== t.id ? 'cursor-grab active:cursor-grabbing' : '',
					selectMode ? 'cursor-pointer hover:border-primary/60' : '',
					isSelected && 'border-primary ring-1 ring-primary/40 bg-primary/5 dark:bg-primary/10',
					dragId === t.id && 'opacity-35'
				)}
			>
				{#if selectMode}
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={cn(
										'size-4 shrink-0 rounded border transition-colors flex items-center justify-center cursor-pointer mt-1',
										isSelected
											? 'border-primary bg-primary text-primary-foreground'
											: 'border-muted-foreground/40 hover:border-foreground bg-background'
									)}
									onclick={(e) => {
										e.stopPropagation();
										toggleSelect(t.id);
									}}
								>
									{#if isSelected}
										<Check class="size-3 stroke-[3]" />
									{/if}
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>{isSelected ? 'Batal tandai' : 'Tandai todo ini'}</Tooltip.Content>
					</Tooltip.Root>
				{:else if singleCat && tab !== 'deleted' && editId !== t.id}
					<GripVertical class="size-3.5 shrink-0 text-muted-foreground/50 mt-1" />
				{/if}

				{#if editId === t.id}
					<!-- svelte-ignore a11y_autofocus -->
					<InputGroup
						class="h-auto min-w-0 flex-1 flex-col rounded-xl border-primary bg-card ring-2 ring-primary/20 overflow-hidden shadow-xs p-1 gap-0.5"
						onfocusout={(e: FocusEvent) => {
							const el = e.currentTarget as HTMLElement;
							const next = e.relatedTarget as Node | null;
							const apply = () => {
								if (editId === null || !el.isConnected) return;
								const cur = tasks.find((x) => x.id === editId);
								const changed =
									!!cur &&
									(editVal.trim() !== cur.text ||
										editDesc.trim() !== (cur.description ?? '') ||
										editCat !== cur.category_id);
								if (changed) {
									confirmDiscard = true;
								} else {
									finishEdit(true);
								}
							};
							if (next instanceof Element) {
								if (el.contains(next) || next.closest('[data-slot="popover-content"]')) return;
								apply();
								return;
							}
							// relatedTarget null: fokus bisa hilang karena elemen internal dilepas (toggle). Tunggu render selesai dulu.
							setTimeout(() => {
								if (editId === null) return;
								const ae = document.activeElement;
								if (ae instanceof Element && ae !== document.body) {
									if (el.contains(ae)) return;
									if (ae.closest('[data-slot="popover-content"],[role="dialog"]')) return;
								}
								apply();
							}, 0);
						}}
						>
							<InputGroupTextarea
								bind:ref={editInput}
								bind:value={editVal}
								rows={1}
								autofocus
								style="field-sizing: content;"
								placeholder="Teks todo"
								onkeydown={onEditKey}
								oninput={(e: Event) => resizeTextarea(e.currentTarget as HTMLTextAreaElement)}
								class="min-h-7 py-1 px-2 text-xs max-h-32 text-foreground leading-normal bg-transparent focus:bg-transparent placeholder:text-muted-foreground/60 border-0 shadow-none resize-none [&]:pt-1 [&]:pb-1"
							/>
							{#if showEditDesc}
								<InputGroupTextarea
									bind:ref={editDescInput}
									bind:value={editDesc}
									rows={1}
									style="field-sizing: content;"
									onkeydown={onEditDescKey}
									oninput={(e: Event) => resizeTextarea(e.currentTarget as HTMLTextAreaElement)}
									placeholder="Tambahkan deskripsi singkat (opsional)"
									class="min-h-6 py-0.5 px-2 text-xs max-h-32 text-muted-foreground leading-normal bg-transparent focus:bg-transparent placeholder:text-muted-foreground/50 border-0 shadow-none [&]:pt-0.5 [&]:pb-0.5"
								/>
							{/if}
							<InputGroupAddon align="block-end" class="!pt-0 !pb-0.5 !px-1 gap-1 bg-transparent border-0 w-full items-center text-xs justify-start">
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<Button
												{...props}
												variant="ghost"
												size="icon-xs"
												onclick={() => {
													showEditDesc = !showEditDesc;
													if (showEditDesc) {
														setTimeout(() => {
															editDescInput?.focus();
															resizeTextarea(editDescInput);
														}, 0);
													}
												}}
												class="size-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md"
											>
												{#if showEditDesc}
													<X />
												{:else}
													<Plus />
												{/if}
											</Button>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>
										{showEditDesc ? 'Sembunyikan Deskripsi' : 'Tambah Deskripsi'}
									</Tooltip.Content>
								</Tooltip.Root>
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<Button
												{...props}
												variant="ghost"
												size="icon-xs"
												onclick={() => insertCopyTag('edit')}
												class="size-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md"
											>
												<Copy class="size-3" />
											</Button>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>Sisipkan [copy]...[/copy] untuk teks clipboard</Tooltip.Content>
								</Tooltip.Root>
								<CategorySelect
									{cats}
									value={editCat}
									placeholder="Kategori"
									class="shrink-0"
									triggerClass="h-6 w-auto max-w-48 border-0 bg-transparent px-2 text-[11px] shadow-none hover:bg-accent/60 rounded-full"
									onSelect={(id) => {
										if (id !== null) editCat = id;
									}}
									{onOpenAddCategory}
								/>
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<InputGroupButton
												{...props}
												variant="default"
												size="icon-xs"
												onclick={() => finishEdit(true)}
												disabled={!editVal.trim()}
												class="size-6 rounded-md ms-auto"
											>
												<Check />
											</InputGroupButton>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>Simpan</Tooltip.Content>
								</Tooltip.Root>
							</InputGroupAddon>
						</InputGroup>
				{:else}
					{@const catItem = byId(t.category_id)}
					{@const catLabel = catItem?.name ?? (t.deleted ? '' : 'Tanpa kategori')}
					<div class="flex min-w-0 flex-1 flex-col gap-1">
						{#if catLabel || t.status_updated_at || t.created_at || (t.subtask_count ?? 0) > 0}
							<div class="flex items-center gap-1.5 flex-wrap min-w-0">
								{#if catLabel}
									<Badge
										variant="outline"
										class={cn(
											'max-w-[70%] whitespace-normal h-auto py-0.5 px-2 text-left text-[10px] font-semibold leading-tight break-words [overflow-wrap:anywhere]',
											getBadgeCls(catItem?.color)
										)}
									>
										{catLabel}
									</Badge>
								{/if}
								{#if (t.subtask_count ?? 0) > 0}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Badge
													{...props}
													variant="outline"
													class={cn(
														'h-auto py-0.5 px-1.5 text-[10px] font-medium leading-tight shrink-0 select-none tabular-nums cursor-pointer flex items-center gap-1 transition-colors',
														t.subtask_done_count === t.subtask_count
															? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950 dark:text-emerald-400 dark:border-emerald-800'
															: 'bg-muted/60 text-muted-foreground border-border/60 hover:bg-muted hover:text-foreground'
													)}
													onclick={(e) => {
														e.stopPropagation();
														if (!selectMode) detailTask = t;
													}}
												>
													<ListChecks class="size-3 shrink-0" />
													<span>{t.subtask_done_count ?? 0}/{t.subtask_count}</span>
												</Badge>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>
											{t.subtask_done_count ?? 0} dari {t.subtask_count} subtask selesai (klik untuk detail)
										</Tooltip.Content>
									</Tooltip.Root>
								{/if}
								{#if t.status_updated_at || t.created_at}
									{@const eventTime = t.status_updated_at || t.created_at}
									{@const dateText = formatTaskDate(eventTime)}
									{#if dateText}
										<Tooltip.Root>
											<Tooltip.Trigger>
												{#snippet child({ props })}
													<Badge
														{...props}
														variant="secondary"
														class="h-auto py-0.5 px-1.5 text-[10px] font-medium leading-tight shrink-0 border border-border/40 bg-muted/60 text-muted-foreground hover:bg-muted select-none tabular-nums cursor-default"
													>
														{dateText}
													</Badge>
												{/snippet}
											</Tooltip.Trigger>
											<Tooltip.Content>
												{getTaskEventTooltip(t.status, t.status_updated_at, t.created_at)}
											</Tooltip.Content>
										</Tooltip.Root>
									{/if}
								{/if}
							</div>
						{/if}
						<div class="flex items-start justify-between gap-1.5 min-w-0">
							<Tooltip.Root
								disabled={selectMode || t.deleted}
							>
								<Tooltip.Trigger class="min-w-0 flex-1 text-left">
									{#snippet child({ props })}
										<!-- svelte-ignore a11y_click_events_have_key_events -->
										<!-- svelte-ignore a11y_no_static_element_interactions -->
										<div
											{...props}
											class={cn(
												'min-w-0 flex flex-col gap-0.5 text-left [overflow-wrap:anywhere] flex-1 cursor-text',
												(t.done || t.status === 'done') && 'text-muted-foreground line-through',
												t.deleted && 'text-muted-foreground'
											)}
											ondblclick={(e) => {
												e.stopPropagation();
												if (!selectMode && !t.deleted) startEdit(t);
											}}
										>
											<span
												use:trackTrunc={{ id: t.id, kind: 't' }}
												class={cn(
													'text-xs select-none transition-all',
													isExpanded ? 'break-words [overflow-wrap:anywhere] whitespace-pre-wrap leading-relaxed' : 'truncate'
												)}
											>
												<CopyableText text={t.text} />
											</span>
											{#if t.description}
												{#if isExpanded}
													<div class="mt-1 rounded-md border bg-muted/40 p-1.5 text-[11px] leading-relaxed text-muted-foreground break-words [overflow-wrap:anywhere] whitespace-pre-wrap select-text">
														<CopyableText text={t.description} />
													</div>
												{:else}
													<button
														type="button"
														use:trackTrunc={{ id: t.id, kind: 'd' }}
														class="truncate text-left text-[10px] leading-tight text-muted-foreground/80 hover:text-foreground cursor-pointer"
														onclick={(e) => {
															e.stopPropagation();
															if (selectMode) return;
															skipEditClose = true;
															startEdit(t);
															setTimeout(() => {
																editDescInput?.focus();
																skipEditClose = false;
															}, 0);
														}}
													>
														<CopyableText text={t.description} />
													</button>
												{/if}
											{/if}
										</div>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Klik 2x untuk edit</Tooltip.Content>
							</Tooltip.Root>

							{#if truncState[t.id]?.t || !!t.description || isExpanded}
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props: chevronProps })}
											<button
												{...chevronProps}
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													toggleExpand(t.id);
												}}
												class="size-5 shrink-0 rounded-md p-0.5 text-muted-foreground/60 hover:text-foreground hover:bg-muted/70 transition-colors inline-flex items-center justify-center cursor-pointer -mt-0.5"
											>
												<ChevronDown class={cn('size-3.5 transition-transform duration-200', isExpanded && 'rotate-180')} />
											</button>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>{isExpanded ? 'Ringkas teks' : 'Perluas teks'}</Tooltip.Content>
								</Tooltip.Root>
							{/if}
						</div>
					</div>

					{#if !selectMode}
						<div class="flex items-center gap-1 shrink-0 mt-0.5">
							{#if tab === 'deleted'}
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<Button {...props} variant="outline" size="icon-xs" class="rounded-md" onclick={() => onRestore(t)}>
												<RotateCcw class="size-3.5" />
											</Button>
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>Pulihkan</Tooltip.Content>
								</Tooltip.Root>

								<DropdownMenu.Root>
									<DropdownMenu.Trigger>
										{#snippet child({ props })}
											<Button {...props} variant="ghost" size="icon-xs" class="rounded-md">
												<Ellipsis class="size-3.5" />
											</Button>
										{/snippet}
									</DropdownMenu.Trigger>
									<DropdownMenu.Content align="end" class="min-w-36 rounded-lg p-1 shadow-lg border border-border">
										<DropdownMenu.Item
											onclick={() => (detailTask = t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<Eye class="size-3.5" /> Lihat Detail
										</DropdownMenu.Item>
										<DropdownMenu.Separator />
										<DropdownMenu.Item
											onclick={() => onHardDelete(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 text-destructive focus:bg-destructive/10 focus:text-destructive cursor-pointer"
										>
											<Trash class="size-3.5" /> Hapus Permanen
										</DropdownMenu.Item>
									</DropdownMenu.Content>
								</DropdownMenu.Root>
							{:else}
								{@const isDone = t.done || t.status === 'done'}
								{@const isInProgress = !t.done && t.status === 'inprogress'}
								{@const isTodo = !t.done && (!t.status || t.status === 'todo')}

								<!-- Status Cycle Button -->
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											{#if isDone}
												<Button
													{...props}
													variant="ghost"
													size="icon-xs"
													class="rounded-md text-muted-foreground hover:text-foreground"
													onclick={(e) => e.stopPropagation()}
													ondblclick={(e) => {
														e.stopPropagation();
														cycleTaskStatus(t);
													}}
												>
													<Undo2 class="size-3.5" />
												</Button>
											{:else if isInProgress}
												<Button
													{...props}
													variant="outline"
													size="icon-xs"
													class="rounded-md border-amber-300 bg-amber-50 text-amber-700 hover:bg-amber-100 hover:border-amber-400 hover:text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-400 dark:hover:bg-amber-900/60"
													onclick={(e) => e.stopPropagation()}
													ondblclick={(e) => {
														e.stopPropagation();
														cycleTaskStatus(t);
													}}
												>
													<CircleCheck class="size-3.5" />
												</Button>
											{:else}
												<Button
													{...props}
													variant="outline"
													size="icon-xs"
													class="rounded-md border-blue-300 bg-blue-50 text-blue-600 hover:bg-blue-100 hover:border-blue-400 hover:text-blue-700 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-400 dark:hover:bg-blue-900/60"
													onclick={(e) => e.stopPropagation()}
													ondblclick={(e) => {
														e.stopPropagation();
														cycleTaskStatus(t);
													}}
												>
													<CircleDashedCheck class="size-3.5" />
												</Button>
											{/if}
										{/snippet}
									</Tooltip.Trigger>
									<Tooltip.Content>
										{isDone ? 'Klik 2x: Kembalikan ke Todo' : isInProgress ? 'Klik 2x: Tandai Selesai' : 'Klik 2x: Mulai In Progress'}
									</Tooltip.Content>
								</Tooltip.Root>

								<!-- Dropdown Menu -->
								<DropdownMenu.Root>
									<DropdownMenu.Trigger>
										{#snippet child({ props })}
											<Button {...props} variant="ghost" size="icon-xs" class="rounded-md">
												<Ellipsis class="size-3.5" />
											</Button>
										{/snippet}
									</DropdownMenu.Trigger>
									<DropdownMenu.Content align="end" class="min-w-40 rounded-lg p-1 shadow-lg border border-border">
										<DropdownMenu.Item
											onclick={() => (detailTask = t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<Eye class="size-3.5" /> Lihat Detail
										</DropdownMenu.Item>
										{#if !isDone}
											<DropdownMenu.Item
												onclick={() => startEdit(t)}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Pencil class="size-3.5" /> Edit Todo
											</DropdownMenu.Item>
										{/if}

										<DropdownMenu.Separator />
										{#if !isTodo}
											<DropdownMenu.Item
												onclick={() => setStatusDirect(t, 'todo')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-blue-600 dark:text-blue-400"
											>
												<Circle class="size-3.5" /> Set ke Todo
											</DropdownMenu.Item>
										{/if}
										{#if !isInProgress}
											<DropdownMenu.Item
												onclick={() => setStatusDirect(t, 'inprogress')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-amber-600 dark:text-amber-400"
											>
												<Clock class="size-3.5" /> Set ke In Progress
											</DropdownMenu.Item>
										{/if}
										{#if !isDone}
											<DropdownMenu.Item
												onclick={() => setStatusDirect(t, 'done')}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-emerald-600 dark:text-emerald-400"
											>
												<Check class="size-3.5" /> Set ke Selesai
											</DropdownMenu.Item>
										{/if}

										<DropdownMenu.Separator />
										{#if onDuplicateTask}
											<DropdownMenu.Item
												onclick={() => onDuplicateTask(t)}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Copy class="size-3.5" /> Duplikat
											</DropdownMenu.Item>
										{/if}
										{#if onDuplicateTask}
											<DropdownMenu.Item
												onclick={() => onDuplicateToCategories?.(t)}
												class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
											>
												<Copy class="size-3.5" /> Duplikat ke Kategori
											</DropdownMenu.Item>
										{/if}
										<DropdownMenu.Item
											onclick={() => onMoveTaskCategory(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<Tags class="size-3.5" /> Ganti Kategori
										</DropdownMenu.Item>
										<DropdownMenu.Item
											onclick={() => openCompare(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<ArrowLeftRight class="size-3.5" /> Bandingkan...
										</DropdownMenu.Item>
										<DropdownMenu.Separator />
										<DropdownMenu.Item
											onclick={() => onSoftDelete(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 text-destructive focus:bg-destructive/10 focus:text-destructive cursor-pointer"
										>
											<Trash class="size-3.5" /> Hapus
										</DropdownMenu.Item>
									</DropdownMenu.Content>
								</DropdownMenu.Root>
							{/if}
						</div>
					{/if}
				{/if}
			</div>
			{#if singleCat && overIdx === view.length && i === view.length - 1}
				<div class="mx-1 mt-1.5 h-0.5 rounded-full bg-primary"></div>
			{/if}
		{/each}
	</main>

	<!-- input todo baru -->
	<footer class="p-2">
		<InputGroup class="h-auto rounded-xl border border-input/60 bg-card shadow-xs transition-colors overflow-hidden p-1 gap-0.5">
			<InputGroupTextarea
				bind:ref={newTextInput}
				bind:value={newText}
				rows={1}
				style="field-sizing: content;"
				placeholder="Tulis todo baru..."
				class="min-h-7 py-1 px-2 text-xs max-h-32 text-foreground leading-normal bg-transparent focus:bg-transparent placeholder:text-muted-foreground/60 border-0 shadow-none resize-none [&]:pt-1 [&]:pb-1"
				oninput={(e: Event) => resizeTextarea(e.currentTarget as HTMLTextAreaElement)}
				onkeydown={(e: KeyboardEvent) => {
					if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
						e.preventDefault();
						submitAdd();
					}
				}}
			/>
			{#if showDesc}
				<InputGroupTextarea
					bind:ref={descInput}
					bind:value={newDesc}
					rows={1}
					style="field-sizing: content;"
					onkeydown={(e: KeyboardEvent) => {
						if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
							e.preventDefault();
							submitAdd();
						}
					}}
					oninput={(e: Event) => resizeTextarea(e.currentTarget as HTMLTextAreaElement)}
					placeholder="Tambahkan deskripsi singkat (opsional)"
					class="min-h-6 py-0.5 px-2 text-xs max-h-32 text-muted-foreground leading-normal bg-transparent focus:bg-transparent placeholder:text-muted-foreground/50 border-0 shadow-none [&]:pt-0.5 [&]:pb-0.5"
				/>
			{/if}
			<InputGroupAddon align="block-end" class="!pt-0 !pb-0.5 !px-1 gap-1 bg-transparent border-0 w-full items-center text-xs justify-start">
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon-xs"
								disabled={false}
								onclick={() => {
									showDesc = !showDesc;
									if (showDesc) {
										setTimeout(() => {
											descInput?.focus();
											resizeTextarea(descInput);
										}, 0);
									}
								}}
								class="size-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md"
							>
								{#if showDesc}
									<X />
								{:else}
									<Plus />
								{/if}
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>
						{showDesc ? 'Sembunyikan Deskripsi' : 'Tambah Deskripsi'}
					</Tooltip.Content>
				</Tooltip.Root>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon-xs"
								onclick={() => insertCopyTag('new')}
								class="size-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded-md"
							>
								<Copy class="size-3" />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Sisipkan [copy]...[/copy] untuk teks clipboard</Tooltip.Content>
				</Tooltip.Root>
				<CategorySelect
					{cats}
					value={newCat}
					placeholder="Kategori"
					class="shrink-0"
					triggerClass="h-6 w-auto max-w-48 border-0 bg-transparent px-2 text-[11px] shadow-none hover:bg-accent/60 rounded-full"
					onSelect={(id) => {
						if (id !== null) newCat = id;
					}}
					{onOpenAddCategory}
				/>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<InputGroupButton
								{...props}
								variant="default"
								size="icon-xs"
								onclick={submitAdd}
								disabled={!newText.trim()}
								class="size-6 rounded-md ms-auto"
							>
								<ArrowUp />
							</InputGroupButton>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Kirim todo</Tooltip.Content>
				</Tooltip.Root>
			</InputGroupAddon>
		</InputGroup>
	</footer>

	<AlertDialog.Root bind:open={confirmDiscard}>
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title class="text-sm font-semibold">Edit belum disimpan</AlertDialog.Title>
				<AlertDialog.Description class="text-xs">Simpan perubahan pada todo ini?</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Action onclick={() => finishEdit(true)}>Simpan</AlertDialog.Action>
				<AlertDialog.Action variant="destructive" onclick={() => finishEdit(false)}>Buang</AlertDialog.Action>
				<AlertDialog.Cancel>Lanjut Edit</AlertDialog.Cancel>
			</AlertDialog.Footer>
		</AlertDialog.Content>
	</AlertDialog.Root>

	<TaskDetailDialog
		bind:task={detailTask}
		{cats}
		onDuplicate={onDuplicateTask}
		onCompare={(t) => openCompare(t)}
		onToggleDone={onToggleDone}
		onSetStatus={onSetTaskStatus}
		onDelete={onSoftDelete}
		onRestore={onRestore}
		onHardDelete={onHardDelete}
		onStartEdit={(t) => startEdit(t)}
		onSubtasksChange={onRefreshTasks}
	/>

	<ProjectDetailDialog
		bind:open={showSpaceDetail}
		project={activeProjectItem ?? null}
		{tasks}
		onEditProject={onOpenEditProject}
		onSetStatus={onSetProjectStatus}
	/>

	<BatchMoveCategoryDialog
		bind:open={batchCategoryDialogOpen}
		count={selectedIds.length}
		{cats}
		onConfirm={handleBatchSetCategoryAction}
	/>

	<TaskCompareDialog
		bind:open={compareOpen}
		bind:taskA={compareTaskA}
		bind:taskB={compareTaskB}
		{tasks}
		{cats}
		onUpdateTask={onUpdateTask}
		onRefreshTasks={onRefreshTasks}
	/>

	<BatchDuplicateToCategoriesDialog
		bind:open={batchDuplicateDialogOpen}
		count={1}
		{cats}
		taskName={duplicateToCategoriesTask?.text ?? ''}
		taskId={duplicateToCategoriesTask?.id ?? 0}
	/>

	<CustomContextMenu
		bind:menuState={contextMenuState}
		{cats}
		{dark}
		projectName={activeProjectItem?.name}
		onFocusNewInput={() => {
			newTextInput?.focus();
		}}
		onOpenDetail={(t) => (detailTask = t)}
		onStartEdit={(t) => startEdit(t)}
		onSetStatus={(id, st) => onSetTaskStatus?.(id, st)}
		onDuplicate={onDuplicateTask}
		onDuplicateToCategories={(t) => {
			duplicateToCategoriesTask = t;
			batchDuplicateDialogOpen = true;
		}}
		onMoveCategory={onMoveTaskCategory}
		onCompare={(t) => openCompare(t)}
		onDelete={onSoftDelete}
		onToggleTheme={onToggleTheme}
		onLockApp={onLockApp}
		onOpenProjectPicker={onOpenProjectPicker}
		onOpenCategoriesView={onOpenCategoriesView}
		onOpenAbout={onOpenAbout}
		onOpenSecuritySettings={onOpenSecuritySettings}
	/>
</div>
