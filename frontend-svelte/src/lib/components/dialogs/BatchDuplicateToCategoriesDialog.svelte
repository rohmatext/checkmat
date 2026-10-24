<script lang="ts">
	import { Search, X, Copy, Check, ChevronDown } from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { getSolidCls, type Category } from '#lib/types.js';
	import { cn } from '#lib/utils';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	let {
		open = $bindable(false),
		count = 0,
		cats = [],
		taskName = '',
		taskId = 0,
		taskIds = [],
		onSuccess
	}: {
		open: boolean;
		count?: number;
		cats: Category[];
		taskName?: string;
		taskId?: number;
		taskIds?: number[];
		onSuccess?: () => void;
	} = $props();

	let query = $state('');
	let selectedCatIds = $state<number[]>([]);
	let showAllSelected = $state(false);
	let isLoading = $state(false);

	$effect(() => {
		if (open) {
			query = '';
			selectedCatIds = [];
			showAllSelected = false;
		}
	});

	const effectiveTaskIds = $derived(
		taskIds && taskIds.length > 0 ? taskIds : taskId && taskId > 0 ? [taskId] : []
	);
	const effectiveCount = $derived(count > 0 ? count : effectiveTaskIds.length);

	const filteredCats = $derived(
		query.trim() === ''
			? cats
			: cats.filter((c) => c.name.toLowerCase().includes(query.trim().toLowerCase()))
	);

	const isAllSelected = $derived(filteredCats.length > 0 && filteredCats.every((c) => selectedCatIds.includes(c.id)));

	async function handleConfirm() {
		if (selectedCatIds.length === 0 || effectiveTaskIds.length === 0) return;
		isLoading = true;
		const totalCats = selectedCatIds.length;
		const totalTasks = effectiveTaskIds.length;
		try {
			if (effectiveTaskIds.length === 1) {
				await invoke('duplicate_task_to_categories', {
					id: effectiveTaskIds[0],
					categoryIds: selectedCatIds
				});
			} else {
				await invoke('duplicate_tasks_to_categories_batch', {
					ids: effectiveTaskIds,
					categoryIds: selectedCatIds
				});
			}
			open = false;
			selectedCatIds = [];
			const taskMsg = totalTasks > 1 ? `${totalTasks} todo` : 'Todo';
			toast.success(`${taskMsg} berhasil diduplikat ke ${totalCats} kategori`);
			onSuccess?.();
		} catch (e) {
			toast.error(String(e));
		} finally {
			isLoading = false;
		}
	}

	function toggleAll() {
		if (isAllSelected) {
			selectedCatIds = selectedCatIds.filter((id) => !filteredCats.some((c) => c.id === id));
		} else {
			selectedCatIds = [...selectedCatIds, ...filteredCats.map((c) => c.id)];
		}
	}

	function toggleCat(id: number) {
		if (selectedCatIds.includes(id)) {
			selectedCatIds = selectedCatIds.filter((x) => x !== id);
		} else {
			selectedCatIds = [...selectedCatIds, id];
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-md gap-3 p-4">
		<Dialog.Header class="gap-1 text-left">
			<Dialog.Title class="text-sm font-semibold flex items-center gap-1.5 flex-wrap break-words [overflow-wrap:anywhere]">
				<Copy class="size-4 text-primary shrink-0" />
				<span>Duplikat ke Kategori ({effectiveCount} Todo)</span>
			</Dialog.Title>
			<Dialog.Description class="text-xs break-words [overflow-wrap:anywhere]">
				{#if effectiveCount === 1 && taskName}
					Pilih kategori tujuan untuk duplikat "<strong>{taskName}</strong>" ({selectedCatIds.length} kategori dipilih):
				{:else}
					Pilih kategori tujuan untuk duplikat {effectiveCount} todo terpilih ({selectedCatIds.length} kategori dipilih):
				{/if}
			</Dialog.Description>
		</Dialog.Header>

		{#if cats.length > 5}
			<div class="relative flex items-center gap-1.5">
				<Search class="absolute left-2.5 size-3.5 text-muted-foreground pointer-events-none" />
				<Input
					bind:value={query}
					placeholder="Cari kategori..."
					class="h-8 pl-8 pr-7 text-xs flex-1"
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
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<button
								{...props}
								type="button"
								class="h-8 px-2.5 text-xs rounded-md border border-input bg-card hover:bg-accent/50 cursor-pointer gap-1.5 flex items-center"
								onclick={() => (showAllSelected = !showAllSelected)}
							>
								<ChevronDown class="size-3.5" />
								<span>{showAllSelected ? 'Sembunyikan' : 'Terpilih'} {selectedCatIds.length}</span>
							</button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>{showAllSelected ? 'Sembunyikan yang dipilih' : 'Tampilkan yang dipilih'}</Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}

		{#if showAllSelected && selectedCatIds.length > 0}
			<div class="flex flex-wrap gap-1 max-h-20 overflow-y-auto p-2 border border-border/50 rounded-md bg-muted/30">
				{#each cats.filter((c) => selectedCatIds.includes(c.id)) as c (c.id)}
					<span class="flex items-center gap-1 px-2 py-0.5 text-[10px] rounded-full bg-primary/10 text-primary border border-primary/20">
						<span class={cn('size-1.5 shrink-0 rounded-full', getSolidCls(c.color))}></span>
						<span class="truncate max-w-[120px]">{c.name}</span>
						<button type="button" class="ml-0.5 hover:text-primary/80" onclick={() => toggleCat(c.id)}>
							<X class="size-2.5" />
						</button>
					</span>
				{/each}
			</div>
		{/if}

		<div class="flex max-h-60 flex-col gap-1.5 overflow-y-auto">
			{#if cats.length > 0 && selectedCatIds.length > 0}
				<div class="flex w-full items-center justify-between gap-2 px-1 py-1 text-[10px] text-muted-foreground border-b border-border/50">
					<span>Pilih kategori (bisa multiple)</span>
					<button
						type="button"
						class="h-5 px-2 text-[10px] rounded border border-input bg-card hover:bg-accent/50 cursor-pointer"
						onclick={toggleAll}
					>
						{isAllSelected ? 'Batalkan semua' : 'Pilih semua'}
					</button>
				</div>
			{/if}
			{#each filteredCats as c (c.id)}
				{@const isSelected = selectedCatIds.includes(c.id)}
				<button
					type="button"
					class={cn(
						'flex w-full items-center justify-between rounded-md px-2.5 py-1.5 text-xs text-left transition-colors border cursor-pointer min-w-0',
						isSelected
							? 'border-primary bg-primary/10 text-foreground font-medium'
							: 'border-border/60 hover:bg-muted/60 text-muted-foreground hover:text-foreground'
					)}
					onclick={() => toggleCat(c.id)}
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
				disabled={selectedCatIds.length === 0 || effectiveTaskIds.length === 0 || isLoading}
				onclick={handleConfirm}
			>
				<Copy class="size-3" />
				{isLoading ? 'Menduplikat...' : `Duplikat ke ${selectedCatIds.length} kategori`}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>