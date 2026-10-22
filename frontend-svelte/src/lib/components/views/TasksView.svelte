<script lang="ts">
	import {
		ArrowLeft,
		FolderOpen,
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
		Eye
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
	import { getBadgeCls, type Category, type Project, type SortDir, type SortField, type Tab, type Task } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		tasks = [],
		cats = [],
		activeProjectItem = undefined,
		activeCat = $bindable(null),
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
		onToggleDone,
		onSetDoneBatch,
		onSoftDelete,
		onSoftDeleteBatch,
		onRestore,
		onRestoreBatch,
		onHardDelete,
		onHardDeleteBatch,
		onMoveTaskCategory,
		onReorderTasks,
		onOpenAbout
	}: {
		tasks: Task[];
		cats: Category[];
		activeProjectItem?: Project;
		activeCat: number | null;
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
		onToggleDone: (t: Task) => void;
		onSetDoneBatch?: (ids: number[], done: boolean, onSuccess?: () => void) => void;
		onSoftDelete: (t: Task) => void;
		onSoftDeleteBatch?: (ids: number[], onSuccess?: () => void) => void;
		onRestore: (t: Task) => Promise<void>;
		onRestoreBatch?: (ids: number[], onSuccess?: () => void) => Promise<void>;
		onHardDelete: (t: Task) => void;
		onHardDeleteBatch?: (ids: number[], onSuccess?: () => void) => void;
		onMoveTaskCategory: (t: Task) => void;
		onReorderTasks: (ids: number[]) => Promise<void>;
		onOpenAbout?: () => void;
	} = $props();

	let newText = $state('');
	let showDesc = $state(false);
	let newDesc = $state('');
	let descInput: HTMLTextAreaElement | null = $state(null);
	let editId = $state<number | null>(null);
	let editVal = $state('');
	let editDesc = $state('');
	let showEditDesc = $state(false);
	let editInput: HTMLInputElement | null = $state(null);
	let editDescInput: HTMLTextAreaElement | null = $state(null);
	let confirmDiscard = $state(false);
	let skipEditClose = $state(false);

	let expandedIds = $state<number[]>([]);
	let detailTask = $state<Task | null>(null);

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

	$effect(() => {
		// Reset pilihan saat tab atau kategori aktif berubah
		tab;
		activeCat;
		selectedIds = [];
	});

	const byId = (id: number | null) => cats.find((c) => c.id === id);
	const curCat = $derived(byId(activeCat));

	const inTab = (t: Task, k: Tab) =>
		k === 'active' ? !t.done && !t.deleted : k === 'done' ? t.done && !t.deleted : t.deleted;

	const count = (k: Tab) =>
		tasks.filter((t) => (activeCat === null || t.category_id === activeCat) && inTab(t, k)).length;

	let rows = $derived.by(() => {
		const filtered = tasks.filter(
			(t) => (activeCat === null || t.category_id === activeCat) && inTab(t, tab)
		);
		if (activeCat !== null) {
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
		activeCat = v === 'all' ? null : Number(v);
		if (activeCat !== null) newCat = activeCat;
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
		showEditDesc = !!t.description;
		skipEditClose = false;
		if (showEditDesc) {
			setTimeout(() => {
				resizeTextarea(editDescInput);
			}, 0);
		}
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
		if (save && val && (val !== t.text || desc !== (t.description ?? ''))) {
			await onUpdateTask(t.id, val, t.category_id, desc || undefined);
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
		await onAddTask(text, catId, desc || undefined);
	}

	/* ----- drag reorder ----- */
	function shift(idx: number) {
		if (activeCat === null) return;
		const cur = dragRows ?? rows;
		const from = cur.findIndex((r) => r.id === dragId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragRows = next;
	}

	function onDragStart(e: DragEvent, t: Task) {
		if (activeCat === null) return;
		dragId = t.id;
		dragRows = [...rows];
		overIdx = rows.findIndex((r) => r.id === t.id);
		e.dataTransfer?.setData('text/plain', String(t.id));
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onRowDragOver(e: DragEvent, i: number) {
		if (activeCat === null) return;
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
		if (activeCat === null) return;
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
		if (activeCat === null) return;
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

	const tabs: { k: Tab; label: string }[] = [
		{ k: 'active', label: 'Todo' },
		{ k: 'done', label: 'Selesai' },
		{ k: 'deleted', label: 'Dihapus' }
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
							onclick={onOpenProjectPicker}
						>
							{activeProjectItem?.name ?? 'Pilih space'}
						</button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Ganti space</Tooltip.Content>
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
	<div class="flex items-center gap-1.5 px-2 pt-2">
		<div class="min-w-0 flex-1">
			<CategorySelect
				{cats}
				value={activeCat}
				allowAll={true}
				allLabel="Semua kategori"
				{tasks}
				onSelect={(id) => handlePickCat(id === null ? 'all' : String(id))}
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
			{#if activeCat !== null}
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
						{#if tab === 'deleted'}
							<Button
								variant="outline"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchRestoreAction}
							>
								<RotateCcw class="size-3" /> Pulihkan
							</Button>
							<Button
								variant="destructive"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchDeleteAction}
							>
								<Trash class="size-3" /> Hapus
							</Button>
						{:else if tab === 'active'}
							<Button
								variant="outline"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1 text-emerald-600 dark:text-emerald-400 border-emerald-500/40 hover:bg-emerald-500/10"
								onclick={handleBatchSetDoneAction}
							>
								<Check class="size-3" /> Selesai
							</Button>
							<Button
								variant="destructive"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchDeleteAction}
							>
								<Trash class="size-3" /> Hapus
							</Button>
						{:else if tab === 'done'}
							<Button
								variant="outline"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1 text-amber-600 dark:text-amber-400 border-amber-500/40 hover:bg-amber-500/10"
								onclick={handleBatchSetUndoneAction}
							>
								<Undo2 class="size-3" /> Ke Todo
							</Button>
							<Button
								variant="destructive"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchDeleteAction}
							>
								<Trash class="size-3" /> Hapus
							</Button>
						{/if}
					{/if}
					<Button
						variant="ghost"
						size="xs"
						class="h-6 px-2 text-[11px] text-muted-foreground hover:text-foreground"
						onclick={exitSelectMode}
					>
						Batal
					</Button>
				</div>
			</div>
		{:else if rows.length > 0}
			<div class="mb-2 flex items-center justify-between gap-1">
				{#if activeCat === null}
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
				{activeCat === null
					? 'Belum ada todo'
					: curCat
						? `Belum ada todo di "${curCat.name}"`
						: 'Belum ada kategori'}
			</p>
		{/if}

		{#each view as t, i (t.id)}
			{@const isSelected = selectedIds.includes(t.id)}
			{@const isExpanded = expandedIds.includes(t.id)}
			{#if activeCat !== null && overIdx === i}
				<div class="mx-1 mb-1 h-0.5 rounded-full bg-primary"></div>
			{/if}
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div
				role="listitem"
				draggable={!selectMode && editId !== t.id && tab !== 'deleted' && activeCat !== null}
				ondragstart={(e) => onDragStart(e, t)}
				ondragover={(e) => onRowDragOver(e, i)}
				ondrop={onDrop}
				ondragend={onDragEnd}
				onclick={() => {
					if (selectMode) toggleSelect(t.id);
				}}
				class={cn(
					'mb-1.5 flex items-center gap-1.5 rounded-lg border bg-card px-2 py-1.5 transition-all select-none',
					activeCat !== null && tab !== 'deleted' && !selectMode && editId !== t.id ? 'cursor-grab active:cursor-grabbing' : '',
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
										'size-4 shrink-0 rounded border transition-colors flex items-center justify-center cursor-pointer',
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
				{:else if activeCat !== null && tab !== 'deleted' && editId !== t.id}
					<GripVertical class="size-3.5 shrink-0 text-muted-foreground/50" />
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
										editDesc.trim() !== (cur.description ?? ''));
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
							<InputGroupInput
								bind:ref={editInput}
								bind:value={editVal}
								autofocus
								autocomplete="off"
								onkeydown={onEditKey}
								class="h-7 min-h-7 px-2 pt-1.5 pb-0.5 text-xs text-foreground placeholder:text-muted-foreground/60 focus:bg-transparent [&]:pt-1.5 [&]:pb-0.5"
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
					<div class="flex min-w-0 flex-1 flex-col gap-1">
						{#if t.deleted}
							<div>
								<Badge
									variant="outline"
									class={cn('max-w-32 truncate text-[10px] font-semibold', getBadgeCls(byId(t.category_id)?.color))}
								>
									{byId(t.category_id)?.name ?? ''}
								</Badge>
							</div>
						{:else}
							<div>
								<Badge
									variant="outline"
									class={cn('max-w-32 truncate text-[10px] font-semibold', getBadgeCls(byId(t.category_id)?.color))}
								>
									{byId(t.category_id)?.name ?? 'Tanpa kategori'}
								</Badge>
							</div>
						{/if}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div
										{...props}
										class={cn(
											'min-w-0 cursor-pointer flex flex-col gap-0.5 text-left [overflow-wrap:anywhere]',
											t.done && 'text-muted-foreground line-through',
											t.deleted && 'text-muted-foreground'
										)}
										onclick={(e) => {
											if (selectMode) return;
											toggleExpand(t.id);
										}}
									>
										<span class={cn('text-xs select-none transition-all', isExpanded ? 'break-words [overflow-wrap:anywhere] whitespace-pre-wrap leading-relaxed' : 'truncate')}>
											{t.text}
										</span>
										{#if t.description}
											{#if isExpanded}
												<div class="mt-1 rounded-md border bg-muted/40 p-1.5 text-[11px] leading-relaxed text-muted-foreground break-words [overflow-wrap:anywhere] whitespace-pre-wrap select-text">
													{t.description}
												</div>
											{:else}
												<button
													type="button"
													class="truncate text-left text-[10px] leading-tight text-muted-foreground/80 hover:text-foreground"
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
													{t.description}
												</button>
											{/if}
										{/if}
									</div>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>{selectMode ? 'Klik untuk menandai' : isExpanded ? 'Klik untuk ringkas' : 'Klik untuk buka'}</Tooltip.Content>
						</Tooltip.Root>
					</div>

					{#if !selectMode}
						{#if tab === 'deleted'}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="outline" size="xs" onclick={() => onRestore(t)}>
											<RotateCcw class="size-3.5" />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Pulihkan</Tooltip.Content>
							</Tooltip.Root>

							<DropdownMenu.Root>
								<DropdownMenu.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="ghost" size="xs">
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
						{:else if tab === 'done'}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="ghost" size="xs" onclick={() => onToggleDone(t)}>
											<Undo2 class="size-3.5" />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Kembalikan ke Todo</Tooltip.Content>
							</Tooltip.Root>

							<DropdownMenu.Root>
								<DropdownMenu.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="ghost" size="xs">
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
									{#if onDuplicateTask}
										<DropdownMenu.Item
											onclick={() => onDuplicateTask(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<Copy class="size-3.5" /> Duplikat
										</DropdownMenu.Item>
									{/if}
									<DropdownMenu.Item
										onclick={() => onMoveTaskCategory(t)}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<Tags class="size-3.5" /> Ganti Kategori
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
						{:else}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button
											{...props}
											variant="outline"
											size="xs"
											onclick={() => onToggleDone(t)}
											class="border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100/70 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-400 dark:hover:bg-emerald-900/60"
										>
											<Check class="size-3.5" />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Tandai selesai</Tooltip.Content>
							</Tooltip.Root>

							<DropdownMenu.Root>
								<DropdownMenu.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="ghost" size="xs">
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
									<DropdownMenu.Item
										onclick={() => startEdit(t)}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<Pencil class="size-3.5" /> Edit Todo
									</DropdownMenu.Item>
									{#if onDuplicateTask}
										<DropdownMenu.Item
											onclick={() => onDuplicateTask(t)}
											class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
										>
											<Copy class="size-3.5" /> Duplikat
										</DropdownMenu.Item>
									{/if}
									<DropdownMenu.Item
										onclick={() => onMoveTaskCategory(t)}
										class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
									>
										<Tags class="size-3.5" /> Ganti Kategori
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
					{/if}
				{/if}
			</div>
			{#if activeCat !== null && overIdx === view.length && i === view.length - 1}
				<div class="mx-1 mt-1.5 h-0.5 rounded-full bg-primary"></div>
			{/if}
		{/each}
	</main>

	<!-- input todo baru -->
	<footer class="p-2">
		<InputGroup class="h-auto rounded-xl border border-input/60 bg-card shadow-xs transition-colors overflow-hidden p-1 gap-0.5">
			<InputGroupInput
				bind:value={newText}
				placeholder="Tulis todo baru..."
				autocomplete="off"
				class="h-7 min-h-7 px-2 pt-1.5 pb-0.5 text-xs text-foreground placeholder:text-muted-foreground/60 focus:bg-transparent [&]:pt-1.5 [&]:pb-0.5"
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
		onToggleDone={onToggleDone}
		onDelete={onSoftDelete}
		onRestore={onRestore}
		onHardDelete={onHardDelete}
		onStartEdit={(t) => startEdit(t)}
	/>
</div>
