<script lang="ts">
	import { Search, X } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
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

	let query = $state('');

	$effect(() => {
		if (task) {
			query = '';
		}
	});

	const filteredCats = $derived(
		query.trim() === ''
			? cats
			: cats.filter((c) => c.name.toLowerCase().includes(query.trim().toLowerCase()))
	);

	async function selectCategory(catId: number) {
		if (!task) return;
		const currentTask = task;
		task = null;
		await onMove(currentTask, catId);
	}
</script>

<Dialog.Root open={task !== null} onOpenChange={(open) => { if (!open) task = null; }}>
	<Dialog.Content class="max-w-xs gap-3 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Ganti kategori</Dialog.Title>
			<Dialog.Description class="text-xs [overflow-wrap:anywhere]">{task?.text ?? ''}</Dialog.Description>
		</Dialog.Header>

		{#if cats.length > 5}
			<div class="relative flex items-center">
				<Search class="absolute left-2.5 size-3.5 text-muted-foreground pointer-events-none" />
				<Input
					bind:value={query}
					placeholder="Cari kategori..."
					class="h-8 pl-8 pr-7 text-xs"
				/>
				{#if query}
					<button
						type="button"
						onclick={() => (query = '')}
						class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
						title="Hapus pencarian"
					>
						<X class="size-3" />
					</button>
				{/if}
			</div>
		{/if}

		<div class="flex max-h-60 flex-col gap-1.5 overflow-y-auto">
			{#each filteredCats as c (c.id)}
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
			{#if filteredCats.length === 0}
				<p class="py-2 text-center text-xs text-muted-foreground">
					{cats.length === 0 ? 'Belum ada kategori.' : 'Kategori tidak ditemukan.'}
				</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Batal</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
