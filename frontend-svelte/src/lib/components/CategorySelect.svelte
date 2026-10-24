<script lang="ts">
	import { Search, Check, ChevronsUpDown, Plus, X, Funnel } from '@lucide/svelte';
	import * as Popover from '#lib/components/ui/popover/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { getSolidCls, type Category, type Task } from '#lib/types.js';
	import { cn } from '#lib/utils.js';

	let {
		cats = [],
		value = null,
		allowAll = false,
		allLabel = 'Semua kategori',
		placeholder = 'Pilih kategori',
		tasks = [],
		multiple = false,
		values = [],
		class: className = '',
		triggerClass = '',
		variant = 'default',
		onSelect,
		onToggle,
		onClear,
		onOpenAddCategory
	}: {
		cats: Category[];
		value?: number | null;
		allowAll?: boolean;
		allLabel?: string;
		placeholder?: string;
		tasks?: Task[];
		multiple?: boolean;
		values?: number[];
		class?: string;
		triggerClass?: string;
		variant?: 'default' | 'filter';
		onSelect?: (val: number | null) => void;
		onToggle?: (id: number) => void;
		onClear?: () => void;
		onOpenAddCategory?: () => void;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let triggerEl: HTMLButtonElement | null = $state(null);
	let popWidth = $state(0);

	$effect(() => {
		if (!triggerEl) return;
		const update = () => {
			popWidth = triggerEl?.getBoundingClientRect().width ?? 0;
		};
		update();
		const ro = new ResizeObserver(update);
		ro.observe(triggerEl);
		return () => ro.disconnect();
	});

	const selectedCat = $derived(cats.find((c) => c.id === value));
	const selectedCats = $derived(cats.filter((c) => values.includes(c.id)));

	const filteredCats = $derived(
		query.trim() === ''
			? cats
			: cats.filter((c) => c.name.toLowerCase().includes(query.trim().toLowerCase()))
	);

	function countFor(catId: number | null): number | null {
		if (!tasks || tasks.length === 0) return null;
		if (catId === null) {
			return tasks.filter((t) => !t.deleted).length;
		}
		return tasks.filter((t) => t.category_id === catId && !t.deleted).length;
	}

	function handlePick(id: number | null) {
		if (multiple) {
			if (id === null) onClear?.();
			else onToggle?.(id);
			query = '';
			return;
		}
		onSelect?.(id);
		open = false;
		query = '';
	}
</script>

<Popover.Root bind:open>
	<Popover.Trigger class={className}>
		{#snippet child({ props })}
			{#if variant === 'filter'}
				<button
					type="button"
					bind:this={triggerEl}
					{...props}
					title={values.length === 0 ? 'Filter Kategori' : `${values.length} kategori dipilih`}
					class={cn(
						'inline-flex h-8 items-center gap-1.5 rounded-4xl border border-input bg-input/30 hover:bg-input/50 px-3 text-xs font-medium shadow-2xs transition-colors focus-visible:outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] cursor-pointer shrink-0',
						triggerClass
					)}
				>
					<Funnel class="size-3.5 shrink-0 opacity-70" />
					<span>Kategori</span>
					{#if values.length > 0}
						<span class="flex items-center justify-center min-w-4 h-4 px-1 rounded-full bg-primary text-[10px] font-bold text-primary-foreground leading-none">
							{values.length}
						</span>
					{/if}
					<ChevronsUpDown class="size-3 shrink-0 opacity-50 -mr-0.5" />
				</button>
			{:else}
				<button
					type="button"
					bind:this={triggerEl}
					{...props}
					title={multiple
						? values.length === 0
							? allLabel
							: `${values.length} kategori dipilih`
						: (selectedCat?.name ?? (value === null && allowAll ? allLabel : placeholder))}
					class={cn(
						'flex h-8 w-full items-center justify-between gap-1.5 rounded-4xl border border-input bg-input/30 hover:bg-input/50 px-3 text-xs font-semibold shadow-2xs transition-colors focus-visible:outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] cursor-pointer',
						triggerClass
					)}
				>
					<span class="flex min-w-0 items-center gap-2 truncate">
						{#if multiple}
							{#if selectedCats.length === 0}
								<span class="size-2 shrink-0 rounded-full bg-primary/60"></span>
								<span class="truncate">{allLabel}</span>
							{:else}
								<span class="size-2 shrink-0 rounded-full bg-primary"></span>
								<span class="truncate">{selectedCats.length === 1
									? selectedCats[0].name
									: `${selectedCats.length} kategori dipilih`}</span>
							{/if}
						{:else if value === null && allowAll}
							<span class="size-2 shrink-0 rounded-full bg-primary/60"></span>
							<span class="truncate">{allLabel}</span>
						{:else if selectedCat}
							<span class={cn('size-2 shrink-0 rounded-full', getSolidCls(selectedCat.color))}></span>
							<span class="truncate">{selectedCat.name}</span>
						{:else}
							<span class="truncate text-muted-foreground">{placeholder}</span>
						{/if}
					</span>
					<ChevronsUpDown class="size-3.5 shrink-0 opacity-50" />
				</button>
			{/if}
		{/snippet}
	</Popover.Trigger>
	<Popover.Content
		class="p-2 gap-2"
		style={popWidth
			? `width:${Math.max(popWidth, 240)}px;min-width:14rem;max-width:calc(100vw - 1rem);`
			: 'min-width:14rem;max-width:calc(100vw - 1rem);'}
		align={variant === 'filter' ? 'end' : 'start'}
	>
		<!-- Search Input -->
		<div class="relative flex items-center">
			<Search class="absolute left-2 size-3.5 text-muted-foreground pointer-events-none" />
			<Input
				bind:value={query}
				placeholder="Cari kategori..."
				class="h-7 pl-7 pr-6 text-xs"
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

		<!-- List items -->
		<div class="flex max-h-52 flex-col gap-0.5 overflow-y-auto pr-0.5">
			{#if allowAll && (query.trim() === '' || allLabel.toLowerCase().includes(query.trim().toLowerCase()))}
				{@const countAll = countFor(null)}
				<button
					type="button"
					class="flex w-full items-start justify-between gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer select-none"
					onclick={() => handlePick(null)}
					title={allLabel}
				>
					<span class="flex items-start gap-2 min-w-0 flex-1">
						<span class="size-2 rounded-full bg-muted-foreground/60 shrink-0 mt-1"></span>
						<span class="line-clamp-2 break-words [overflow-wrap:anywhere] whitespace-normal leading-snug">{allLabel}</span>
					</span>
					<span class="flex items-center gap-1.5 shrink-0 text-muted-foreground mt-0.5">
						{#if countAll !== null}
							<span class="text-[10px]">{countAll}</span>
						{/if}
						{#if multiple ? values.length === 0 : value === null}
							<Check class="size-3.5 text-foreground stroke-2" />
						{/if}
					</span>
				</button>
			{/if}

			{#each filteredCats as c (c.id)}
				{@const countCat = countFor(c.id)}
				{@const isSelected = multiple ? values.includes(c.id) : value === c.id}
				<button
					type="button"
					class="flex w-full items-start justify-between gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground transition-colors hover:bg-accent hover:text-accent-foreground cursor-pointer select-none"
					onclick={() => handlePick(c.id)}
					title={c.name}
				>
					<span class="flex items-start gap-2 min-w-0 flex-1">
						<span class={cn('size-2 shrink-0 rounded-full mt-1', getSolidCls(c.color))}></span>
						<span class="line-clamp-2 break-words [overflow-wrap:anywhere] whitespace-normal leading-snug">{c.name}</span>
					</span>
					<span class="flex items-center gap-1.5 shrink-0 text-muted-foreground mt-0.5">
						{#if countCat !== null}
							<span class="text-[10px]">{countCat}</span>
						{/if}
						{#if isSelected}
							<Check class="size-3.5 text-foreground stroke-2" />
						{/if}
					</span>
				</button>
			{/each}

			{#if filteredCats.length === 0 && (!allowAll || (query && !allLabel.toLowerCase().includes(query.trim().toLowerCase())))}
				<div class="py-4 text-center text-xs text-muted-foreground">
					Kategori tidak ditemukan
				</div>
			{/if}
		</div>

		{#if onOpenAddCategory}
			<div class="border-t border-border pt-1">
				<Button
					variant="ghost"
					size="sm"
					class="h-7 w-full justify-start gap-1.5 px-2 text-xs text-muted-foreground hover:text-accent-foreground"
					onclick={() => {
						open = false;
						onOpenAddCategory();
					}}
				>
					<Plus class="size-3.5" />
					Tambah Kategori Baru
				</Button>
			</div>
		{/if}
	</Popover.Content>
</Popover.Root>
