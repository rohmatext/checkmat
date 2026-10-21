<script lang="ts">
	import { Copy, Pencil, Check, Undo2, RotateCcw, Trash, X } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { getBadgeCls, type Category, type Task } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		task = $bindable<Task | null>(null),
		cats = [],
		onDuplicate,
		onToggleDone,
		onDelete,
		onRestore,
		onHardDelete,
		onStartEdit
	}: {
		task: Task | null;
		cats: Category[];
		onDuplicate?: (t: Task) => void;
		onToggleDone?: (t: Task) => void;
		onDelete?: (t: Task) => void;
		onRestore?: (t: Task) => void;
		onHardDelete?: (t: Task) => void;
		onStartEdit?: (t: Task) => void;
	} = $props();

	const category = $derived(cats.find((c) => c.id === task?.category_id));

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
				{:else if task?.done}
					<Badge variant="outline" class="border-emerald-300 bg-emerald-50 text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-400 text-[10px]">
						Selesai
					</Badge>
				{:else}
					<Badge variant="outline" class="border-primary/30 bg-primary/10 text-primary text-[10px]">
						Aktif
					</Badge>
				{/if}
			</div>
			<Dialog.Title class="text-base font-semibold leading-snug break-words [overflow-wrap:anywhere] select-text">
				{task?.text ?? ''}
			</Dialog.Title>
		</Dialog.Header>

		{#if task?.description}
			<div class="space-y-1">
				<span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
					Deskripsi
				</span>
				<div class="max-h-48 overflow-y-auto rounded-lg border bg-muted/30 p-3 text-xs leading-relaxed text-foreground select-text whitespace-pre-wrap break-words [overflow-wrap:anywhere]">
					{task.description}
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
