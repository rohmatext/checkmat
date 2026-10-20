<script lang="ts">
	import { ArrowLeft, Plus, ArrowUp, ArrowDown, Pencil, Trash, GripVertical } from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { PALETTE, getBadgeCls, getSolidCls, type Category, type Project, type Task } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		cats = [],
		tasks = [],
		activeProjectItem = undefined,
		onBack,
		onOpenAddCategory,
		onOpenRenameCategory,
		onDeleteCategory,
		onMoveCategory,
		onReorderCategories,
		onSetColorQuick
	}: {
		cats: Category[];
		tasks: Task[];
		activeProjectItem?: Project;
		onBack: () => void;
		onOpenAddCategory: () => void;
		onOpenRenameCategory: (cat: Category) => void;
		onDeleteCategory: (cat: Category) => void;
		onMoveCategory: (index: number, direction: -1 | 1) => Promise<void>;
		onReorderCategories?: (ids: number[]) => Promise<void>;
		onSetColorQuick: (cat: Category, color: string) => Promise<void>;
	} = $props();

	let dragCatId = $state<number | null>(null);
	let overCatIdx = $state<number | null>(null);
	let dragCats = $state<Category[] | null>(null);

	let viewCats = $derived(dragCats ?? cats);

	function shiftCat(idx: number) {
		const cur = dragCats ?? cats;
		const from = cur.findIndex((r) => r.id === dragCatId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragCats = next;
	}

	function onDragStart(e: DragEvent, c: Category) {
		dragCatId = c.id;
		dragCats = [...cats];
		overCatIdx = cats.findIndex((r) => r.id === c.id);
		e.dataTransfer?.setData('text/plain', String(c.id));
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onRowDragOver(e: DragEvent, i: number) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const mid = rect.top + rect.height / 2;
		const insertIdx = e.clientY < mid ? i : i + 1;
		if (insertIdx !== overCatIdx) {
			overCatIdx = insertIdx;
			shiftCat(insertIdx);
		}
	}

	function onMainDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const items = document.querySelectorAll('[data-cat-item]');
		if (items.length === 0) return;
		const last = items[items.length - 1].getBoundingClientRect();
		if (e.clientY > last.bottom && overCatIdx !== items.length) {
			overCatIdx = items.length;
			shiftCat(items.length);
		}
	}

	async function onDrop(e: DragEvent) {
		e.preventDefault();
		const final = dragCats;
		dragCatId = null;
		overCatIdx = null;
		dragCats = null;
		if (!final) return;
		const currentIds = cats.map((c) => c.id);
		const newIds = final.map((c) => c.id);
		if (newIds.every((id, idx) => id === currentIds[idx])) return;
		if (onReorderCategories) {
			await onReorderCategories(newIds);
		}
	}

	function onDragEnd() {
		dragCatId = null;
		overCatIdx = null;
		dragCats = null;
	}
</script>

<div class="flex h-full flex-col p-3">
	<header class="flex items-center justify-between gap-2 pb-3">
		<div class="flex items-center gap-2 min-w-0">
			<Button variant="outline" size="icon-sm" title="Kembali ke tugas" onclick={onBack}>
				<ArrowLeft />
			</Button>
			<div class="min-w-0">
				<h1 class="text-base font-bold truncate">Kelola Kategori</h1>
				<p class="text-xs text-muted-foreground truncate">{activeProjectItem?.name ?? 'Project'}</p>
			</div>
		</div>
		<Button size="sm" onclick={onOpenAddCategory}>
			<Plus /> Kategori
		</Button>
	</header>

	<main
		class="flex-1 space-y-2 overflow-y-auto pr-0.5"
		ondragover={onMainDragOver}
		ondrop={onDrop}
	>
		{#if cats.length === 0}
			<div class="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
				Belum ada kategori. Buat kategori untuk mulai mengelompokkan todo.
			</div>
		{/if}
		{#each viewCats as c, idx (c.id)}
			{@const activeCount = tasks.filter((t) => t.category_id === c.id && !t.deleted && !t.done).length}
			{@const totalCount = tasks.filter((t) => t.category_id === c.id && !t.deleted).length}
			{@const isDragging = c.id === dragCatId}
			<div
				data-cat-item
				role="listitem"
				draggable={true}
				ondragstart={(e) => onDragStart(e, c)}
				ondragover={(e) => onRowDragOver(e, idx)}
				ondrop={onDrop}
				ondragend={onDragEnd}
				class={cn(
					'flex items-center gap-2 rounded-lg border bg-card p-2.5 shadow-xs transition-all duration-150',
					isDragging && 'opacity-40 border-primary ring-2 ring-primary/40 bg-accent/40 shadow-md'
				)}
			>
				<!-- Grip Handle Drag & Drop -->
				<div
					class="cursor-grab active:cursor-grabbing text-muted-foreground/50 hover:text-foreground touch-none select-none p-0.5 -ml-0.5"
					title="Geser posisi (Drag & Drop)"
				>
					<GripVertical class="size-4" />
				</div>

				<!-- Tombol panah urutan naik / turun alternatif -->
				<div class="flex flex-col gap-0.5">
					<Button
						variant="ghost"
						size="icon-xs"
						class="h-4 w-4 p-0 text-muted-foreground hover:text-foreground disabled:opacity-30"
						disabled={idx === 0}
						onclick={() => onMoveCategory(idx, -1)}
						title="Geser ke atas"
					>
						<ArrowUp class="size-3" />
					</Button>
					<Button
						variant="ghost"
						size="icon-xs"
						class="h-4 w-4 p-0 text-muted-foreground hover:text-foreground disabled:opacity-30"
						disabled={idx === cats.length - 1}
						onclick={() => onMoveCategory(idx, 1)}
						title="Geser ke bawah"
					>
						<ArrowDown class="size-3" />
					</Button>
				</div>

				<!-- Badge & nama kategori -->
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-1.5 flex-wrap">
						<span class={cn('inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold', getBadgeCls(c.color))}>
							<span class={cn('size-2 rounded-full', getSolidCls(c.color))}></span>
							<span class="max-w-[130px] truncate">{c.name}</span>
						</span>
						<span class="text-[11px] text-muted-foreground">
							{totalCount} todo ({activeCount} aktif)
						</span>
					</div>

					<!-- Palet warna cepat -->
					<div class="mt-2 flex items-center gap-1">
						{#each PALETTE as tone}
							<button
								type="button"
								class={cn(
									'size-3.5 rounded-full transition-transform cursor-pointer',
									getSolidCls(tone),
									c.color === tone ? 'ring-2 ring-primary ring-offset-1 scale-110' : 'opacity-60 hover:opacity-100 hover:scale-105'
								)}
								onclick={() => onSetColorQuick(c, tone)}
								title={`Ubah warna ke ${tone}`}
							></button>
						{/each}
					</div>
				</div>

				<!-- Action edit & delete -->
				<div class="flex items-center gap-1">
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="outline"
									size="icon-sm"
									title="Edit nama kategori"
									onclick={() => onOpenRenameCategory(c)}
								>
									<Pencil />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Ganti nama & warna</Tooltip.Content>
					</Tooltip.Root>

					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="outline"
									size="icon-sm"
									disabled={cats.length <= 1}
									title="Hapus kategori"
									class="hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
									onclick={() => onDeleteCategory(c)}
								>
									<Trash />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>{cats.length <= 1 ? 'Minimal harus ada satu kategori' : 'Hapus kategori'}</Tooltip.Content>
					</Tooltip.Root>
				</div>
			</div>
		{/each}
	</main>

	<footer class="pt-2 text-[11px] text-muted-foreground text-center">
		Tarik (drag & drop) atau gunakan panah untuk mengatur urutan kategori.
	</footer>
</div>
