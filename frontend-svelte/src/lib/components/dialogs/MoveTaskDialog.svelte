<script lang="ts">
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { getSolidCls, type Category, type Task } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		task = $bindable<Task | null>(null),
		cats = [],
		onMove
	}: {
		task: Task | null;
		cats: Category[];
		onMove: (t: Task, catId: number) => Promise<void>;
	} = $props();

	async function selectCategory(catId: number) {
		if (!task) return;
		const currentTask = task;
		task = null;
		await onMove(currentTask, catId);
	}
</script>

<Dialog.Root open={task !== null} onOpenChange={(open) => { if (!open) task = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Ganti kategori</Dialog.Title>
			<Dialog.Description class="text-xs [overflow-wrap:anywhere]">{task?.text ?? ''}</Dialog.Description>
		</Dialog.Header>
		<div class="flex max-h-60 flex-col gap-1.5 overflow-y-auto">
			{#each cats as c (c.id)}
				<Button
					variant={task?.category_id === c.id ? 'secondary' : 'outline'}
					size="sm"
					class="justify-start gap-2"
					onclick={() => selectCategory(c.id)}
				>
					<span class={cn('size-2 rounded-full', getSolidCls(c.color))}></span>
					<span class="truncate">{c.name}</span>
				</Button>
			{/each}
			{#if cats.length === 0}
				<p class="py-2 text-center text-xs text-muted-foreground">Belum ada kategori.</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Batal</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
