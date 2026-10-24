<script lang="ts">
	import { Copy, Pencil, Check, Undo2, RotateCcw, Trash, X, Clock, ArrowLeftRight, Plus, GripVertical } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import {
		InputGroup,
		InputGroupAddon,
		InputGroupButton,
		InputGroupInput
	} from '#lib/components/ui/input-group/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { formatFullDate, getBadgeCls, getTaskEventTooltip, type Category, type Task, type TaskEvent, type TaskStatus, type Subtask } from '#lib/types.js';
	import CopyableText from '#lib/components/CopyableText.svelte';
	import { cn } from '#lib/utils';

	let {
		task = $bindable<Task | null>(null),
		cats = [],
		onDuplicate,
		onDuplicateToCategories,
		onToggleDone,
		onSetStatus,
		onDelete,
		onRestore,
		onHardDelete,
		onStartEdit,
		onCompare,
		onSubtasksChange
	}: {
		task: Task | null;
		cats: Category[];
		onDuplicate?: (t: Task) => void;
		onDuplicateToCategories?: (t: Task) => void;
		onToggleDone?: (t: Task) => void;
		onSetStatus?: (id: number, status: TaskStatus) => void;
		onDelete?: (t: Task) => void;
		onRestore?: (t: Task) => void;
		onHardDelete?: (t: Task) => void;
		onStartEdit?: (t: Task) => void;
		onCompare?: (t: Task) => void;
		onSubtasksChange?: () => void;
	} = $props();

	const category = $derived(cats.find((c) => c.id === task?.category_id));
	let events = $state<TaskEvent[]>([]);
	let subtasks = $state<Subtask[]>([]);
	let newSubtaskTitle = $state('');
	let editingSubtaskId = $state<number | null>(null);
	let editingSubtaskTitle = $state('');

	/* ----- drag reorder subtasks ----- */
	let dragSubtaskId = $state<number | null>(null);
	let overSubtaskIdx = $state<number | null>(null);
	let dragSubtasks = $state<Subtask[] | null>(null);

	function shiftSubtask(idx: number) {
		const cur = dragSubtasks ?? subtasks;
		const from = cur.findIndex((s) => s.id === dragSubtaskId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragSubtasks = next;
	}

	function onSubtaskDragStart(e: DragEvent, st: Subtask) {
		if (editingSubtaskId !== null || task?.deleted) return;
		dragSubtaskId = st.id;
		dragSubtasks = [...subtasks];
		overSubtaskIdx = subtasks.findIndex((s) => s.id === st.id);
		e.dataTransfer?.setData('text/plain', String(st.id));
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onSubtaskDragOver(e: DragEvent, i: number) {
		if (dragSubtaskId === null) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const mid = rect.top + rect.height / 2;
		const insertIdx = e.clientY < mid ? i : i + 1;
		if (insertIdx !== overSubtaskIdx) {
			overSubtaskIdx = insertIdx;
			shiftSubtask(insertIdx);
		}
	}

	function onSubtaskListDragOver(e: DragEvent) {
		if (dragSubtaskId === null) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const items = document.querySelectorAll('[data-subtask-item]');
		if (items.length === 0) return;
		const last = items[items.length - 1].getBoundingClientRect();
		if (e.clientY > last.bottom && overSubtaskIdx !== items.length) {
			overSubtaskIdx = items.length;
			shiftSubtask(items.length);
		}
	}

	async function onSubtaskDrop(e: DragEvent) {
		e.preventDefault();
		const final = dragSubtasks;
		dragSubtaskId = null;
		overSubtaskIdx = null;
		dragSubtasks = null;
		if (!final || !task) return;
		const currentIds = subtasks.map((s) => s.id);
		const newIds = final.map((s) => s.id);
		if (newIds.every((id, idx) => id === currentIds[idx])) return;
		subtasks = final;
		try {
			await invoke('reorder_subtasks', { taskId: task.id, ids: newIds });
			onSubtasksChange?.();
		} catch (err) {
			console.error('Gagal reorder subtasks:', err);
		}
	}

	function onSubtaskDragEnd() {
		dragSubtaskId = null;
		overSubtaskIdx = null;
		dragSubtasks = null;
	}

	const subtaskCount = $derived(subtasks.length);
	const subtaskDoneCount = $derived(subtasks.filter((s) => s.done).length);
	const progressPct = $derived(subtaskCount > 0 ? Math.round((subtaskDoneCount / subtaskCount) * 100) : 0);

	$effect(() => {
		dragSubtaskId = null;
		overSubtaskIdx = null;
		dragSubtasks = null;
		if (task) {
			invoke<TaskEvent[]>('list_task_events', { taskId: task.id })
				.then((res) => {
					events = res || [];
				})
				.catch(() => {
					events = [];
				});

			invoke<Subtask[]>('list_subtasks', { taskId: task.id })
				.then((res) => {
					subtasks = res || [];
				})
				.catch(() => {
					subtasks = [];
				});
		} else {
			events = [];
			subtasks = [];
			newSubtaskTitle = '';
			editingSubtaskId = null;
			editingSubtaskTitle = '';
		}
	});

	async function handleAddSubtask() {
		if (!task || task.deleted) return;
		const title = newSubtaskTitle.trim();
		if (!title) return;
		try {
			const added = await invoke<Subtask>('add_subtask', { taskId: task.id, title });
			subtasks = [...subtasks, added];
			newSubtaskTitle = '';
			if (task) {
				task.subtask_count = (task.subtask_count ?? 0) + 1;
			}
			onSubtasksChange?.();
		} catch (e) {
			console.error('Gagal menambah subtask:', e);
		}
	}

	async function handleToggleSubtask(st: Subtask) {
		if (!task) return;
		const newDone = !st.done;
		subtasks = subtasks.map((s) => (s.id === st.id ? { ...s, done: newDone } : s));
		try {
			await invoke('set_subtask_done', { id: st.id, done: newDone });
			if (task) {
				task.subtask_done_count = subtasks.filter((s) => s.done).length;
			}
			onSubtasksChange?.();
		} catch (e) {
			console.error('Gagal mengubah status subtask:', e);
		}
	}

	async function handleDeleteSubtask(stId: number) {
		if (!task) return;
		subtasks = subtasks.filter((s) => s.id !== stId);
		if (editingSubtaskId === stId) {
			editingSubtaskId = null;
			editingSubtaskTitle = '';
		}
		try {
			await invoke('delete_subtask', { id: stId });
			if (task) {
				task.subtask_count = subtasks.length;
				task.subtask_done_count = subtasks.filter((s) => s.done).length;
			}
			onSubtasksChange?.();
		} catch (e) {
			console.error('Gagal menghapus subtask:', e);
		}
	}

	function handleStartEditSubtask(st: Subtask) {
		editingSubtaskId = st.id;
		editingSubtaskTitle = st.title;
	}

	async function handleSaveEditSubtask(st: Subtask) {
		if (editingSubtaskId !== st.id) return;
		const trimmed = editingSubtaskTitle.trim();
		editingSubtaskId = null;
		if (!trimmed || trimmed === st.title) {
			editingSubtaskTitle = '';
			return;
		}
		subtasks = subtasks.map((s) => (s.id === st.id ? { ...s, title: trimmed } : s));
		editingSubtaskTitle = '';
		try {
			await invoke('update_subtask_title', { id: st.id, title: trimmed });
			onSubtasksChange?.();
		} catch (e) {
			console.error('Gagal update judul subtask:', e);
		}
	}

	function handleDuplicate() {
		if (!task || !onDuplicate || task.deleted) return;
		const cur = task;
		onDuplicate(cur);
	}

	function handleDuplicateToCategories() {
		if (!task || !onDuplicateToCategories || task.deleted) return;
		const cur = task;
		onDuplicateToCategories(cur);
	}

	function handleCompare() {
		if (!task || !onCompare) return;
		const cur = task;
		task = null;
		onCompare(cur);
	}

	function handleToggleDone() {
		if (!task || !onToggleDone || task.deleted) return;
		const cur = task;
		onToggleDone(cur);
		task = null;
	}

	function handleDelete() {
		if (!task || !onDelete || task.deleted) return;
		const cur = task;
		task = null;
		onDelete(cur);
	}

	function handleRestore() {
		if (!task || !onRestore) return;
		const cur = task;
		task = null;
		onRestore(cur);
	}

	function handleHardDelete() {
		if (!task || !onHardDelete) return;
		const cur = task;
		task = null;
		onHardDelete(cur);
	}

	function handleEdit() {
		if (!task || !onStartEdit || task.deleted) return;
		const cur = task;
		task = null;
		onStartEdit(cur);
	}
</script>

<Dialog.Root open={task !== null} onOpenChange={(open) => { if (!open) task = null; }}>
	<Dialog.Content class="max-w-md gap-4 p-5">
		<Dialog.Header class="gap-1.5 text-left">
			<div class="flex items-center gap-1.5 flex-wrap min-w-0">
				{#if category}
					<Badge
						variant="outline"
						class={cn('max-w-40 truncate text-[11px] font-semibold', getBadgeCls(category.color))}
					>
						{category.name}
					</Badge>
				{/if}
				{#if task?.deleted}
					<Badge variant="outline" class="border-destructive/30 bg-destructive/10 text-destructive text-[10px]">
						Dihapus
					</Badge>
				{:else if task?.done || task?.status === 'done'}
					<Badge variant="outline" class="border-emerald-300 bg-emerald-50 text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-400 text-[10px]">
						Selesai
					</Badge>
				{:else if task?.status === 'inprogress'}
					<Badge variant="outline" class="border-amber-300 bg-amber-50 text-amber-700 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-400 text-[10px]">
						In Progress
					</Badge>
				{:else}
					<Badge variant="outline" class="border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-800 dark:bg-blue-950 dark:text-blue-400 text-[10px]">
						Todo
					</Badge>
				{/if}
				{#if task?.status_updated_at || task?.created_at}
					<span class="text-[10px] text-muted-foreground/70 break-words [overflow-wrap:anywhere]">
						· {getTaskEventTooltip(task.status, task.status_updated_at, task.created_at)}
					</span>
				{/if}
			</div>
			<Dialog.Title class="text-base font-semibold leading-snug break-words [overflow-wrap:anywhere] select-text">
				<CopyableText text={task?.text ?? ''} />
			</Dialog.Title>
		</Dialog.Header>

		{#if task?.description}
			<div class="space-y-1">
				<span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
					Deskripsi
				</span>
				<div class="max-h-48 overflow-y-auto rounded-lg border bg-muted/30 p-3 text-xs leading-relaxed text-foreground select-text whitespace-pre-wrap break-words [overflow-wrap:anywhere]">
					<CopyableText text={task.description} />
				</div>
			</div>
		{/if}

		<!-- Subtasks Section -->
		<div class="space-y-2 pt-1 border-t border-border/60">
			<div class="flex items-center justify-between gap-2 flex-wrap">
				<div class="flex items-center gap-1.5">
					<span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
						Subtask
					</span>
					{#if subtaskCount > 0}
						<span class="text-[10px] text-muted-foreground/80 font-mono">
							({subtaskDoneCount}/{subtaskCount})
						</span>
					{/if}
				</div>
				{#if subtaskCount > 0}
					<span class="text-[10px] text-muted-foreground font-medium">
						{progressPct}% selesai
					</span>
				{/if}
			</div>

			{#if subtaskCount > 0}
				<div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
					<div
						class="h-full bg-emerald-500 rounded-full transition-all duration-300"
						style="width: {progressPct}%"
					></div>
				</div>

				<div
					role="list"
					class="space-y-1 max-h-44 overflow-y-auto rounded-lg border bg-muted/20 p-1.5"
					ondragover={onSubtaskListDragOver}
					ondrop={onSubtaskDrop}
				>
					{#each dragSubtasks ?? subtasks as st, idx (st.id)}
						{#if overSubtaskIdx === idx}
							<div class="mx-1 h-0.5 rounded-full bg-primary"></div>
						{/if}
						<div
							data-subtask-item
							role="listitem"
							draggable={editingSubtaskId === null && !task?.deleted}
							ondragstart={(e) => onSubtaskDragStart(e, st)}
							ondragover={(e) => onSubtaskDragOver(e, idx)}
							ondrop={onSubtaskDrop}
							ondragend={onSubtaskDragEnd}
							class={cn(
								'group flex items-center justify-between gap-1.5 px-1.5 py-1 rounded hover:bg-muted/40 transition-colors min-w-0 select-none',
								editingSubtaskId === null && !task?.deleted && 'cursor-grab active:cursor-grabbing',
								dragSubtaskId === st.id && 'opacity-40 bg-accent/40 ring-1 ring-primary/40'
							)}
						>
							<div class="flex items-center gap-1.5 min-w-0 flex-1">
								{#if !task?.deleted && editingSubtaskId === null}
									<GripVertical class="size-3 text-muted-foreground/40 group-hover:text-muted-foreground/80 shrink-0 touch-none" />
								{/if}

								<button
									type="button"
									onclick={() => handleToggleSubtask(st)}
									class={cn(
										'size-4 shrink-0 rounded border transition-colors flex items-center justify-center cursor-pointer',
										st.done
											? 'border-emerald-500 bg-emerald-500 text-white'
											: 'border-muted-foreground/40 hover:border-foreground bg-background'
									)}
								>
									{#if st.done}
										<Check class="size-3 stroke-[3]" />
									{/if}
								</button>

								{#if editingSubtaskId === st.id}
									<!-- svelte-ignore a11y_autofocus -->
									<input
										type="text"
										bind:value={editingSubtaskTitle}
										onkeydown={(e) => {
											if (e.key === 'Enter') handleSaveEditSubtask(st);
											if (e.key === 'Escape') {
												editingSubtaskId = null;
												editingSubtaskTitle = '';
											}
										}}
										onblur={() => handleSaveEditSubtask(st)}
										class="flex-1 bg-background border border-primary rounded px-1.5 py-0.5 text-xs text-foreground focus:outline-none"
										autofocus
									/>
								{:else}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<span
										class={cn(
											'text-xs select-text break-words [overflow-wrap:anywhere] flex-1 cursor-text leading-snug',
											st.done && 'line-through text-muted-foreground'
										)}
										ondblclick={() => handleStartEditSubtask(st)}
									>
										{st.title}
									</span>
								{/if}
							</div>

							<div class="flex items-center gap-0.5 shrink-0 opacity-80 group-hover:opacity-100">
								{#if editingSubtaskId === st.id}
									<Button
										variant="ghost"
										size="icon-xs"
										class="size-5 rounded"
										onclick={() => handleSaveEditSubtask(st)}
									>
										<Check class="size-3" />
									</Button>
								{:else}
									<Button
										variant="ghost"
										size="icon-xs"
										class="size-5 rounded text-muted-foreground hover:text-foreground"
										onclick={() => handleStartEditSubtask(st)}
									>
										<Pencil class="size-3" />
									</Button>
									<Button
										variant="ghost"
										size="icon-xs"
										class="size-5 rounded text-muted-foreground hover:text-destructive"
										onclick={() => handleDeleteSubtask(st.id)}
									>
										<Trash class="size-3" />
									</Button>
								{/if}
							</div>
						</div>
					{/each}
					{#if overSubtaskIdx === (dragSubtasks ?? subtasks).length}
						<div class="mx-1 h-0.5 rounded-full bg-primary"></div>
					{/if}
				</div>
			{/if}

			{#if !task?.deleted}
				<InputGroup class="h-8 rounded-lg border border-input/60 bg-muted/20 shadow-none px-1 transition-colors focus-within:border-primary/70 focus-within:bg-background">
					<InputGroupInput
						bind:value={newSubtaskTitle}
						onkeydown={(e) => {
							if (e.key === 'Enter') {
								e.preventDefault();
								handleAddSubtask();
							}
						}}
						placeholder="Tambah subtask... (Enter)"
						class="h-7 text-xs px-2 py-0 placeholder:text-muted-foreground/60"
					/>
					<InputGroupAddon align="inline-end" class="!pr-0.5 !py-0 gap-1">
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<InputGroupButton
										{...props}
										variant="secondary"
										size="xs"
										disabled={!newSubtaskTitle.trim()}
										onclick={handleAddSubtask}
										class="h-6 gap-1 px-2 text-xs rounded-md font-medium cursor-pointer"
									>
										<Plus class="size-3" />
										<span>Tambah</span>
									</InputGroupButton>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Tambah subtask (Enter)</Tooltip.Content>
						</Tooltip.Root>
					</InputGroupAddon>
				</InputGroup>
			{/if}
		</div>

		{#if !task?.deleted && onSetStatus}
			<div class="flex items-center justify-between gap-2 pt-1 border-t border-border/60 flex-wrap">
				<span class="text-[11px] text-muted-foreground shrink-0">Ubah status:</span>
				<div class="flex gap-1 flex-wrap">
					<button
						type="button"
						onclick={() => {
							if (task) {
								onSetStatus(task.id, 'todo');
								task = null;
							}
						}}
						class={cn(
							'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
							!task?.done && (!task?.status || task?.status === 'todo')
								? 'bg-blue-600 text-white font-semibold'
								: 'bg-muted hover:bg-muted/80 text-muted-foreground'
						)}
					>
						Todo
					</button>
					<button
						type="button"
						onclick={() => {
							if (task) {
								onSetStatus(task.id, 'inprogress');
								task = null;
							}
						}}
						class={cn(
							'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
							!task?.done && task?.status === 'inprogress'
								? 'bg-amber-600 text-white font-semibold'
								: 'bg-muted hover:bg-muted/80 text-muted-foreground'
						)}
					>
						In Progress
					</button>
					<button
						type="button"
						onclick={() => {
							if (task) {
								onSetStatus(task.id, 'done');
								task = null;
							}
						}}
						class={cn(
							'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
							task?.done || task?.status === 'done'
								? 'bg-emerald-600 text-white font-semibold'
								: 'bg-muted hover:bg-muted/80 text-muted-foreground'
						)}
					>
						Selesai
					</button>
				</div>
			</div>
		{/if}
 
 		{#if events.length > 0}
 			<div class="space-y-1.5 pt-1 border-t border-border/60">
 				<span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
 					Riwayat Status
 				</span>
 				<div class="space-y-1.5 max-h-32 overflow-y-auto rounded-lg border bg-muted/20 p-2 text-[11px]">
 					{#each events as ev, idx (ev.id)}
 						<div class="flex items-center justify-between text-muted-foreground gap-2 min-w-0 flex-wrap">
 							<div class="flex items-center gap-1.5 min-w-0 flex-1">
 								<span
 									class={cn(
 										'size-1.5 rounded-full shrink-0',
 										ev.status === 'done'
 											? 'bg-emerald-500'
 											: ev.status === 'inprogress'
 												? 'bg-amber-500'
 												: 'bg-blue-500'
 									)}
 								></span>
 								<span class="font-medium text-foreground break-words [overflow-wrap:anywhere]">
 									{#if idx === 0}
 										Dibuat ({ev.status === 'done' ? 'Selesai' : ev.status === 'inprogress' ? 'In Progress' : 'Todo'})
 									{:else}
 										Ubah ke {ev.status === 'done' ? 'Selesai' : ev.status === 'inprogress' ? 'In Progress' : 'Todo'}
 									{/if}
 								</span>
 							</div>
 							<span class="tabular-nums text-[10px] text-muted-foreground/80 shrink-0">
 								{formatFullDate(ev.created_at)}
 							</span>
 						</div>
 					{/each}
 				</div>
 			</div>
 		{/if}
 
 		<Dialog.Footer class="flex items-center justify-between gap-2 pt-2 flex-wrap sm:justify-between">
			<div class="flex items-center gap-1.5 flex-wrap">
				{#if task?.deleted}
					{#if onRestore}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleRestore}
										class="gap-1"
									>
										<RotateCcw class="size-3" />
										<span>Pulihkan</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Pulihkan todo</Tooltip.Content>
						</Tooltip.Root>
					{/if}
					{#if onHardDelete}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="destructive"
										size="xs"
										onclick={handleHardDelete}
										class="gap-1"
									>
										<Trash class="size-3" />
										<span>Hapus Permanen</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Hapus permanen</Tooltip.Content>
						</Tooltip.Root>
					{/if}
				{:else}
					{#if onStartEdit}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleEdit}
										class="gap-1"
									>
										<Pencil class="size-3" />
										<span>Edit</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Edit todo</Tooltip.Content>
						</Tooltip.Root>
					{/if}

					{#if onDuplicate}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleDuplicate}
										class="gap-1"
									>
										<Copy class="size-3" />
										<span>Duplikat</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Buat salinan todo ini</Tooltip.Content>
						</Tooltip.Root>
					{/if}

					{#if onDuplicateToCategories}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleDuplicateToCategories}
										class="gap-1"
									>
										<Copy class="size-3" />
										<span>Duplikat ke Kategori</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Duplikat todo ini ke beberapa kategori</Tooltip.Content>
						</Tooltip.Root>
					{/if}

					{#if onCompare}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleCompare}
										class="gap-1"
									>
										<ArrowLeftRight class="size-3" />
										<span>Bandingkan</span>
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Bandingkan dengan todo lain</Tooltip.Content>
						</Tooltip.Root>
					{/if}

					{#if onToggleDone}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="outline"
										size="xs"
										onclick={handleToggleDone}
										class={task?.done ? 'gap-1' : 'gap-1 border-emerald-200 bg-emerald-50 text-emerald-700 hover:bg-emerald-100/70 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-400'}
									>
										{#if task?.done}
											<Undo2 class="size-3" />
											<span>Batal Selesai</span>
										{:else}
											<Check class="size-3" />
											<span>Selesai</span>
										{/if}
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>{task?.done ? 'Kembalikan ke belum selesai' : 'Tandai selesai'}</Tooltip.Content>
						</Tooltip.Root>
					{/if}

					{#if onDelete}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="destructive"
										size="xs"
										onclick={handleDelete}
									>
										<Trash class="size-3" />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Hapus</Tooltip.Content>
						</Tooltip.Root>
					{/if}
				{/if}
			</div>

			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'xs' })}>
				Tutup
			</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
