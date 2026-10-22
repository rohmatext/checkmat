<script lang="ts">
	import { Search, X, FolderInput, Check } from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { getSolidCls, type Category } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		open = $bindable(false),
		count = 0,
		cats = [],
		onConfirm
	}: {
		open: boolean;
		count: number;
		cats: Category[];
		onConfirm: (catId: number) => void;
	} = $props();

	let query = $state('');
	let selectedCatId = $state<number | null>(null);

	$effect(() => {
		if (open) {
			query = '';
			selectedCatId = null;
		}
	});

	const filteredCats = $derived(
		query.trim() === ''
			? cats
			: cats.filter((c) => c.name.toLowerCase().includes(query.trim().toLowerCase()))
	);

	function handleConfirm() {
		if (selectedCatId === null) return;
		const id = selectedCatId;
		open = false;
		onConfirm(id);
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-xs gap-3 p-4">
		<Dialog.Header class="gap-1 text-left">
			<Dialog.Title class="text-sm font-semibold flex items-center gap-1.5 flex-wrap break-words [overflow-wrap:anywhere]">
				<FolderInput class="size-4 text-primary shrink-0" />
				<span>Ganti Kategori ({count} Todo)</span>
			</Dialog.Title>
			<Dialog.Description class="text-xs break-words [overflow-wrap:anywhere]">
				Pilih kategori tujuan untuk {count} todo terpilih:
			</Dialog.Description>
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
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									onclick={() => (query = '')}
									class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
								>
									<X class="size-3" />
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Hapus pencarian</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			</div>
		{/if}

		<div class="flex max-h-60 flex-col gap-1.5 overflow-y-auto">
			{#each filteredCats as c (c.id)}
				{@const isSelected = selectedCatId === c.id}
				<button
					type="button"
					class={cn(
						'flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-xs text-left transition-colors border cursor-pointer min-w-0',
						isSelected
							? 'border-primary bg-primary/10 text-foreground font-medium'
							: 'border-border/60 hover:bg-muted/60 text-muted-foreground hover:text-foreground'
					)}
					onclick={() => (selectedCatId = c.id)}
				>
					<div class="flex items-center gap-2 truncate min-w-0 flex-1">
						<span class={cn('size-2 shrink-0 rounded-full', getSolidCls(c.color))}></span>
						<span class="truncate">{c.name}</span>
					</div>
					{#if isSelected}
						<Check class="size-3.5 text-primary shrink-0 ml-1" />
					{/if}
				</button>
			{/each}
			{#if filteredCats.length === 0}
				<p class="py-2 text-center text-xs text-muted-foreground">
					{cats.length === 0 ? 'Belum ada kategori.' : 'Kategori tidak ditemukan.'}
				</p>
			{/if}
		</div>

		<Dialog.Footer class="gap-1.5 pt-1 flex-wrap">
			<Button variant="outline" size="sm" class="h-8 text-xs" onclick={() => (open = false)}>
				Batal
			</Button>
			<Button
				variant="default"
				size="sm"
				class="h-8 text-xs gap-1.5"
				disabled={selectedCatId === null}
				onclick={handleConfirm}
			>
				Pindahkan
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
