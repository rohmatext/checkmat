<script lang="ts">
	import { Copy, Pencil, Check, Undo2, RotateCcw, Trash, X, Clock } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { formatFullDate, getBadgeCls, getTaskEventTooltip, type Category, type Task, type TaskEvent, type TaskStatus } from '#lib/types.js';
	import CopyableText from '#lib/components/CopyableText.svelte';
	import { cn } from '#lib/utils';

	let {
		task = $bindable<Task | null>(null),
		cats = [],
		onDuplicate,
		onToggleDone,
		onSetStatus,
		onDelete,
		onRestore,
		onHardDelete,
		onStartEdit
	}: {
		task: Task | null;
		cats: Category[];
		onDuplicate?: (t: Task) => void;
		onToggleDone?: (t: Task) => void;
		onSetStatus?: (id: number, status: TaskStatus) => void;
		onDelete?: (t: Task) => void;
		onRestore?: (t: Task) => void;
		onHardDelete?: (t: Task) => void;
		onStartEdit?: (t: Task) => void;
	} = $props();

	const category = $derived(cats.find((c) => c.id === task?.category_id));
	let events = $state<TaskEvent[]>([]);

	$effect(() => {
		if (task) {
			invoke<TaskEvent[]>('list_task_events', { taskId: task.id })
				.then((res) => {
					events = res || [];
				})
				.catch(() => {
					events = [];
				});
		} else {
			events = [];
		}
	});

	function handleDuplicate() {
		if (!task || !onDuplicate || task.deleted) return;
		const cur = task;
		onDuplicate(cur);
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
			<div class="flex items-center gap-2">
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
					<span class="text-[10px] text-muted-foreground/70 shrink-0">
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

		{#if !task?.deleted && onSetStatus}
			<div class="flex items-center justify-between gap-1 pt-1 border-t border-border/60">
				<span class="text-[11px] text-muted-foreground">Ubah status:</span>
				<div class="flex gap-1">
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
 						<div class="flex items-center justify-between text-muted-foreground gap-2">
 							<div class="flex items-center gap-1.5 min-w-0">
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
 								<span class="font-medium text-foreground truncate">
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
 
 		<Dialog.Footer class="flex flex-row items-center justify-between gap-2 pt-2 sm:justify-between">
			<div class="flex items-center gap-1.5">
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
