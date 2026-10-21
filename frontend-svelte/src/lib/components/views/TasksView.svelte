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
		ListChecks
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
		onToggleDone,
		onSoftDelete,
		onSoftDeleteBatch,
		onRestore,
		onRestoreBatch,
		onHardDelete,
		onHardDeleteBatch,
		onMoveTaskCategory,
		onReorderTasks
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
		onToggleDone: (t: Task) => void;
		onSoftDelete: (t: Task) => void;
		onSoftDeleteBatch?: (ids: number[]) => void;
		onRestore: (t: Task) => Promise<void>;
		onRestoreBatch?: (ids: number[]) => Promise<void>;
		onHardDelete: (t: Task) => void;
		onHardDeleteBatch?: (ids: number[]) => void;
		onMoveTaskCategory: (t: Task) => void;
		onReorderTasks: (ids: number[]) => Promise<void>;
	} = $props();

	let newText = $state('');
	let showDesc = $state(false);
	let newDesc = $state('');
	let descInput: HTMLTextAreaElement | null = $state(null);
	let editId = $state<number | null>(null);
	let editVal = $state('');
	let editDesc = $state('');
	let editCat = $state<number | null>(null);
	let showEditDesc = $state(false);
	let editInput: HTMLInputElement | null = $state(null);
	let editDescInput: HTMLTextAreaElement | null = $state(null);
	let confirmDiscard = $state(false);
	let skipEditClose = $state(false);

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
				return 'Nama tugas';
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
				onHardDeleteBatch(ids);
			}
		} else {
			if (onSoftDeleteBatch) {
				onSoftDeleteBatch(ids);
			}
		}
		selectedIds = [];
	}

	async function handleBatchRestoreAction() {
		if (selectedIds.length === 0) return;
		const ids = [...selectedIds];
		if (onRestoreBatch) {
			await onRestoreBatch(ids);
		}
		selectedIds = [];
	}

	function startEdit(t: Task) {
		if (t.deleted) return;
		editId = t.id;
		editVal = t.text;
		editDesc = t.description ?? '';
		editCat = t.category_id;
		showEditDesc = !!t.description;
		skipEditClose = false;
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
		{ k: 'active', label: 'Task' },
		{ k: 'done', label: 'Selesai' },
		{ k: 'deleted', label: 'Dihapus' }
	];
</script>

<div class="flex h-full flex-col">
	<!-- project title and navigation + app utilities -->
	<div class="flex items-center justify-between gap-1 px-2 pt-2">
		<div class="flex min-w-0 flex-1 items-center gap-1">
			<Button variant="outline" size="icon-sm" title="Kembali ke space" onclick={onBackToProjects}>
				<ArrowLeft />
			</Button>
			<button
				type="button"
				class="min-w-0 flex-1 truncate px-2 text-left text-sm font-bold hover:text-primary cursor-pointer"
				title="Pilih space lain"
				onclick={onOpenProjectPicker}
			>
				{activeProjectItem?.name ?? 'Pilih space'}
			</button>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="ghost" size="icon-sm" class="text-muted-foreground hover:text-foreground shrink-0" onclick={onOpenProjectPicker} title="Pilih space">
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
						<Button {...props} variant="outline" size="icon-sm" title={dark ? 'Mode terang' : 'Mode gelap'} onclick={onToggleTheme}>
							{#if dark}<Sun />{:else}<Moon />{/if}
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>{dark ? 'Mode terang' : 'Mode gelap'}</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" title="Kunci aplikasi" onclick={onLockApp}>
							<Lock />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Kunci aplikasi</Tooltip.Content>
			</Tooltip.Root>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="outline" size="icon-sm" title="Menu opsi">
							<EllipsisVertical />
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="w-44">
					<DropdownMenu.Item onclick={onOpenImportDialog} class="cursor-pointer">
						<Upload class="mr-2 size-4" /> Import CSV
					</DropdownMenu.Item>
					<DropdownMenu.Item onclick={onOpenExportDialog} class="cursor-pointer">
						<Download class="mr-2 size-4" /> Export CSV
					</DropdownMenu.Item>
					<DropdownMenu.Separator />
					<DropdownMenu.Item onclick={onOpenCategoriesView} class="cursor-pointer">
						<Tags class="mr-2 size-4" /> Kelola Kategori
					</DropdownMenu.Item>
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
						<Button {...props} variant="outline" size="icon-sm" title="Kelola kategori" onclick={onOpenCategoriesView}>
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
							<Button {...props} variant="outline" size="icon-sm" title="Opsi kategori">
								<Ellipsis />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end" class="w-44">
						<DropdownMenu.Item onclick={onOpenRenameCategory} class="cursor-pointer">
							<Pencil class="mr-2 size-4" /> Edit Kategori
						</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item
							onclick={onDeleteCategory}
							class="text-destructive focus:bg-destructive/10 focus:text-destructive cursor-pointer"
						>
							<Trash class="mr-2 size-4" /> Hapus Kategori
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			{/if}
		</div>
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
		<!-- Batch Toolbar atau Normal Baris Urutan / Tombol Pilih -->
		{#if selectMode}
			<div class="mb-2 flex items-center justify-between gap-1.5 rounded-lg border border-primary/30 bg-primary/10 px-2 py-1.5 text-xs animate-in fade-in-50">
				<div class="flex items-center gap-2 min-w-0">
					<button
						type="button"
						class={cn(
							'size-4 shrink-0 rounded border transition-colors flex items-center justify-center cursor-pointer',
							allRowsSelected
								? 'border-primary bg-primary text-primary-foreground'
								: 'border-muted-foreground/40 hover:border-foreground bg-background'
						)}
						onclick={toggleSelectAll}
						title={allRowsSelected ? 'Batal pilih semua' : 'Pilih semua'}
					>
						{#if allRowsSelected}
							<Check class="size-3 stroke-[3]" />
						{/if}
					</button>
					<span class="text-[11px] font-semibold truncate">
						{selectedIds.length} dipilih
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
								title="Pulihkan yang dipilih"
							>
								<RotateCcw class="size-3" /> Pulihkan
							</Button>
							<Button
								variant="destructive"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchDeleteAction}
								title="Hapus permanen yang dipilih"
							>
								<Trash class="size-3" /> Hapus
							</Button>
						{:else}
							<Button
								variant="destructive"
								size="xs"
								class="h-6 px-2 text-[11px] gap-1"
								onclick={handleBatchDeleteAction}
								title="Hapus yang dipilih"
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
						Selesai
					</Button>
				</div>
			</div>
		{:else if rows.length > 0}
			<div class="mb-2 flex items-center justify-between gap-1">
				{#if activeCat === null}
					<div class="flex items-center gap-1 rounded-md border border-input/40 bg-muted/30 px-2 py-0.5 text-xs">
						<span class="flex items-center gap-1 text-[11px] font-medium text-muted-foreground">
							<ArrowUpDown class="size-3 text-primary" />
							Urutan:
						</span>
						<Select.Root
							type="single"
							value={sortField}
							onValueChange={(v) => v && (sortField = v as SortField)}
						>
							<Select.Trigger
								size="sm"
								class="h-6 w-fit gap-1 rounded-md border border-input/60 bg-background/50 px-2 text-[11px] font-medium shadow-none hover:bg-muted"
							>
								<Select.Value>
									{#snippet children()}
										{sortFieldLabel(sortField)}
									{/snippet}
								</Select.Value>
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="cat_pos" label="Kategori + Todo Posisi" />
								<Select.Item value="pos" label="Todo Posisi" />
								<Select.Item value="category" label="Nama Kategori" />
								<Select.Item value="id" label="Waktu dibuat" />
								<Select.Item value="text" label="Nama tugas" />
							</Select.Content>
						</Select.Root>
						<Button
							variant="outline"
							size="icon-xs"
							class="size-6 border-input/60 bg-background/50 hover:bg-muted"
							title={sortDir === 'asc' ? 'Urutan Naik (A-Z / Lama ke Baru)' : 'Urutan Turun (Z-A / Baru ke Lama)'}
							onclick={toggleSortDir}
						>
							{#if sortDir === 'asc'}
								<ArrowUp class="size-3 text-primary" />
							{:else}
								<ArrowDown class="size-3 text-primary" />
							{/if}
						</Button>
					</div>
				{:else}
					<div class="text-[11px] text-muted-foreground px-1">
						{rows.length} tugas
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
								title="Tandai beberapa todo untuk hapus batch"
							>
								<ListChecks class="size-3.5 text-primary" />
								<span>Pilih</span>
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Tandai banyak tugas untuk dihapus sekaligus</Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}

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
			{@const isSelected = selectedIds.includes(t.id)}
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
					activeCat !== null && tab !== 'deleted' && !selectMode ? 'cursor-grab active:cursor-grabbing' : '',
					selectMode ? 'cursor-pointer hover:border-primary/60' : '',
					isSelected && 'border-primary ring-1 ring-primary/40 bg-primary/5 dark:bg-primary/10',
					dragId === t.id && 'opacity-35'
				)}
			>
				{#if selectMode}
					<button
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
						title={isSelected ? 'Batalkan pilihan' : 'Pilih todo ini'}
					>
						{#if isSelected}
							<Check class="size-3 stroke-[3]" />
						{/if}
					</button>
				{:else if activeCat !== null && tab !== 'deleted'}
					<GripVertical class="size-3.5 shrink-0 text-muted-foreground/50" />
				{/if}

				{#if editId === t.id}
					<!-- svelte-ignore a11y_autofocus -->
					<InputGroup
						class="h-auto min-w-0 flex-1 flex-col rounded-xl border-primary bg-card ring-3 ring-primary/15"
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
							<InputGroupInput
								bind:ref={editInput}
								bind:value={editVal}
								autofocus
								autocomplete="off"
								onkeydown={onEditKey}
								class="h-7 text-xs"
							/>
							{#if showEditDesc}
								<InputGroupTextarea
									bind:ref={editDescInput}
									bind:value={editDesc}
									rows={1}
									onkeydown={onEditKey}
									placeholder="Tambahkan deskripsi singkat (opsional)"
									class="text-[10px] text-muted-foreground"
								/>
							{/if}
							<InputGroupAddon align="block-end" class="gap-1.5 border-t bg-transparent">
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<Button
												{...props}
												variant="ghost"
												size="icon-xs"
												onclick={() => {
													showEditDesc = !showEditDesc;
													if (showEditDesc) setTimeout(() => editDescInput?.focus(), 0);
												}}
												class="shrink-0 text-muted-foreground hover:text-primary"
												title={showEditDesc ? 'Sembunyikan Deskripsi' : 'Tambah Deskripsi'}
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
								<CategorySelect
									{cats}
									value={editCat}
									placeholder="Kategori"
									class="shrink-0"
									triggerClass="h-6 max-w-28 border-0 bg-transparent px-2 text-xs shadow-none hover:bg-accent/40 rounded-full"
									onSelect={(id) => {
										if (id !== null) editCat = id;
									}}
									{onOpenAddCategory}
								/>
								<InputGroupButton
									variant="default"
									size="icon-xs"
									title="Simpan"
									onclick={() => finishEdit(true)}
									disabled={!editVal.trim()}
									class="ms-auto"
								>
									<ArrowUp />
								</InputGroupButton>
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
								<Tooltip.Root>
									<Tooltip.Trigger>
										{#snippet child({ props })}
											<button
												{...props}
												type="button"
												onclick={(e) => {
													if (!selectMode) {
														e.stopPropagation();
														onMoveTaskCategory(t);
													}
												}}
												class="max-w-32 cursor-pointer"
											>
												<Badge
													variant="outline"
													class={cn('max-w-32 truncate text-[10px] font-semibold hover:opacity-75', getBadgeCls(byId(t.category_id)?.color))}
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
									<div
										{...props}
										class={cn(
											'min-w-0 cursor-default flex flex-col gap-0.5 text-left [overflow-wrap:anywhere]',
											t.done && 'text-muted-foreground line-through',
											t.deleted && 'text-muted-foreground'
										)}
										ondblclick={() => {
											if (!selectMode) startEdit(t);
										}}
									>
										<span class="truncate">{t.text}</span>
										{#if t.description}
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
												title="Edit deskripsi"
											>
												{t.description}
											</button>
										{/if}
									</div>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>{selectMode ? 'Klik untuk memilih' : 'Klik 2x untuk edit'}</Tooltip.Content>
						</Tooltip.Root>
					</div>

					{#if !selectMode}
						{#if tab === 'deleted'}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="outline" size="xs" onclick={() => onRestore(t)}>
											<RotateCcw />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Pulihkan</Tooltip.Content>
							</Tooltip.Root>
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="destructive" size="xs" onclick={() => onHardDelete(t)}>
											<Trash />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Hapus permanen</Tooltip.Content>
							</Tooltip.Root>
						{:else if tab === 'done'}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="ghost" size="xs" onclick={() => onToggleDone(t)}>
											<Undo2 />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Kembalikan ke Task</Tooltip.Content>
							</Tooltip.Root>
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="destructive" size="xs" onclick={() => onSoftDelete(t)}>
											<Trash />
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
											onclick={() => onToggleDone(t)}
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
										<Button {...props} variant="destructive" size="xs" onclick={() => onSoftDelete(t)}>
											<Trash />
										</Button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Hapus</Tooltip.Content>
							</Tooltip.Root>
						{/if}
					{/if}
				{/if}
			</div>
			{#if activeCat !== null && overIdx === view.length && i === view.length - 1}
				<div class="mx-1 mt-1.5 h-0.5 rounded-full bg-primary"></div>
			{/if}
		{/each}
	</main>

	<!-- input tugas baru -->
	<footer class="p-2">
		<InputGroup class="h-auto rounded-2xl bg-card">
			<InputGroupInput
				bind:value={newText}
				placeholder="Tulis tugas baru..."
				autocomplete="off"
				class="h-7 text-xs"
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
					rows={2}
					placeholder="Tambahkan deskripsi singkat (opsional)"
					class="text-[10px] text-muted-foreground"
				/>
			{/if}
			<InputGroupAddon align="block-end" class="gap-1.5 border-t bg-transparent">
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon-xs"
								disabled={false}
								onclick={() => (showDesc = !showDesc)}
								class="shrink-0 text-muted-foreground hover:text-primary"
								title={showDesc ? 'Sembunyikan Deskripsi' : 'Tambah Deskripsi'}
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
					triggerClass="h-6 max-w-28 border-0 bg-transparent px-2 text-xs shadow-none hover:bg-accent/40 rounded-full"
					onSelect={(id) => {
						if (id !== null) newCat = id;
					}}
					{onOpenAddCategory}
				/>
				<InputGroupButton
					variant="default"
					size="icon-xs"
					title="Kirim tugas"
					onclick={submitAdd}
					disabled={!newText.trim()}
					class="ms-auto"
				>
					<ArrowUp />
				</InputGroupButton>
			</InputGroupAddon>
		</InputGroup>
	</footer>

	<AlertDialog.Root bind:open={confirmDiscard}>
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title>Edit belum disimpan</AlertDialog.Title>
				<AlertDialog.Description>Simpan perubahan pada tugas ini?</AlertDialog.Description>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Action onclick={() => finishEdit(true)}>Simpan</AlertDialog.Action>
				<AlertDialog.Action variant="destructive" onclick={() => finishEdit(false)}>Buang</AlertDialog.Action>
				<AlertDialog.Cancel>Lanjut Edit</AlertDialog.Cancel>
			</AlertDialog.Footer>
		</AlertDialog.Content>
	</AlertDialog.Root>
</div>
