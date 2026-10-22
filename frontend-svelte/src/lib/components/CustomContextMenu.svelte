<script lang="ts">
	import {
		CircleCheck,
		CircleDashedCheck,
		Check,
		Copy,
		Eye,
		FolderOpen,
		Lock,
		Moon,
		Pencil,
		Plus,
		Sun,
		Tags,
		Trash,
		ArrowLeftRight,
		Info,
		Layers,
		ShieldCheck
	} from '@lucide/svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { getBadgeCls, type Category, type Task, type TaskStatus } from '#lib/types.js';
	import { cn } from '#lib/utils.js';
	import { toast } from 'svelte-sonner';

	let {
		menuState = $bindable<{
			type: 'task' | 'background';
			task?: Task;
			x: number;
			y: number;
		} | null>(null),
		cats = [],
		dark = false,
		projectName = 'Space Saya',
		onFocusNewInput,
		onOpenDetail,
		onStartEdit,
		onSetStatus,
		onDuplicate,
		onMoveCategory,
		onCompare,
		onDelete,
		onToggleTheme,
		onLockApp,
		onOpenProjectPicker,
		onOpenCategoriesView,
		onOpenAbout,
		onOpenSecuritySettings
	}: {
		menuState: {
			type: 'task' | 'background';
			task?: Task;
			x: number;
			y: number;
		} | null;
		cats: Category[];
		dark: boolean;
		projectName?: string;
		onFocusNewInput?: () => void;
		onOpenDetail?: (t: Task) => void;
		onStartEdit?: (t: Task) => void;
		onSetStatus?: (id: number, status: TaskStatus) => void;
		onDuplicate?: (t: Task) => void;
		onMoveCategory?: (t: Task) => void;
		onCompare?: (t: Task) => void;
		onDelete?: (t: Task) => void;
		onToggleTheme?: () => void;
		onLockApp?: () => void;
		onOpenProjectPicker?: () => void;
		onOpenCategoriesView?: () => void;
		onOpenAbout?: () => void;
		onOpenSecuritySettings?: () => void;
	} = $props();

	let adjustedX = $state(0);
	let adjustedY = $state(0);

	$effect(() => {
		if (menuState) {
			const rawX = menuState.x;
			const rawY = menuState.y;
			const menuW = 200;
			const menuH = menuState.type === 'task' ? 340 : 220;
			const winW = typeof window !== 'undefined' ? window.innerWidth : 480;
			const winH = typeof window !== 'undefined' ? window.innerHeight : 600;

			adjustedX = rawX + menuW > winW - 10 ? Math.max(10, winW - menuW - 10) : rawX;
			adjustedY = rawY + menuH > winH - 10 ? Math.max(10, winH - menuH - 10) : rawY;
		}
	});

	function close() {
		menuState = null;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}

	async function copyToClipboard(text: string, label: string) {
		try {
			await navigator.clipboard.writeText(text);
			toast.success(`${label} disalin`);
		} catch {
			toast.error('Gagal menyalin');
		}
		close();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if menuState}
	<!-- Backdrop untuk mendeteksi klik luar atau klik kanan lain -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 bg-transparent"
		onclick={close}
		oncontextmenu={(e) => {
			e.preventDefault();
			close();
		}}
	></div>

	<!-- Menu Card -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		role="menu"
		tabindex="-1"
		class="fixed z-50 w-52 rounded-xl border border-border/80 bg-popover/95 p-1 text-popover-foreground shadow-2xl backdrop-blur-md transition-all select-none"
		style="left: {adjustedX}px; top: {adjustedY}px;"
		onclick={(e) => e.stopPropagation()}
	>
		{#if menuState.type === 'task' && menuState.task}
			{@const t = menuState.task}
			{@const isDone = t.done || t.status === 'done'}
			{@const isInProgress = !t.done && t.status === 'inprogress'}
			{@const isTodo = !t.done && (!t.status || t.status === 'todo')}
			{@const cat = cats.find((c) => c.id === t.category_id)}

			<!-- Header Todo Info -->
			<div class="px-2 py-1.5 border-b border-border/60">
				<p class="text-xs font-semibold text-foreground truncate leading-tight" title={t.text}>
					{t.text}
				</p>
				<div class="flex items-center gap-1.5 mt-1 flex-wrap">
					{#if cat}
						<Badge variant="outline" class={cn('text-[9px] px-1 py-0 h-3.5', getBadgeCls(cat.color))}>
							{cat.name}
						</Badge>
					{/if}
					<span
						class={cn(
							'text-[9px] px-1 py-0 rounded border font-medium',
							isDone
								? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30'
								: isInProgress
									? 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30'
									: 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/30'
						)}
					>
						{isDone ? 'Selesai' : isInProgress ? 'In Progress' : 'Todo'}
					</span>
				</div>
			</div>

			<!-- Quick Status Actions -->
			<div class="py-1 border-b border-border/60">
				<div class="px-2 py-0.5 text-[9px] font-bold text-muted-foreground uppercase tracking-wider">
					Ubah Status
				</div>
				{#if !isTodo}
					<button
						type="button"
						class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
						onclick={() => {
							onSetStatus?.(t.id, 'todo');
							close();
						}}
					>
						<CircleDashedCheck class="size-3.5 text-blue-500" />
						<span>Set ke Todo</span>
					</button>
				{/if}
				{#if !isInProgress}
					<button
						type="button"
						class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
						onclick={() => {
							onSetStatus?.(t.id, 'inprogress');
							close();
						}}
					>
						<CircleCheck class="size-3.5 text-amber-500" />
						<span>Set ke In Progress</span>
					</button>
				{/if}
				{#if !isDone}
					<button
						type="button"
						class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
						onclick={() => {
							onSetStatus?.(t.id, 'done');
							close();
						}}
					>
						<Check class="size-3.5 text-emerald-500" />
						<span>Set ke Selesai</span>
					</button>
				{/if}
			</div>

			<!-- Task Actions -->
			<div class="py-1 border-b border-border/60">
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onOpenDetail?.(t);
						close();
					}}
				>
					<Eye class="size-3.5 text-muted-foreground" />
					<span>Lihat Detail</span>
				</button>
				{#if !isDone}
					<button
						type="button"
						class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
						onclick={() => {
							onStartEdit?.(t);
							close();
						}}
					>
						<Pencil class="size-3.5 text-muted-foreground" />
						<span>Edit Todo</span>
					</button>
				{/if}
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onCompare?.(t);
						close();
					}}
				>
					<ArrowLeftRight class="size-3.5 text-muted-foreground" />
					<span>Bandingkan Todo</span>
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onDuplicate?.(t);
						close();
					}}
				>
					<Copy class="size-3.5 text-muted-foreground" />
					<span>Duplikat Todo</span>
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onMoveCategory?.(t);
						close();
					}}
				>
					<Tags class="size-3.5 text-muted-foreground" />
					<span>Ganti Kategori</span>
				</button>
			</div>

			<!-- Clipboard & Delete -->
			<div class="pt-1">
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => copyToClipboard(t.text, 'Teks todo')}
				>
					<Copy class="size-3.5 text-muted-foreground" />
					<span>Salin Teks</span>
				</button>
				{#if t.description}
					<button
						type="button"
						class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
						onclick={() => copyToClipboard(t.description ?? '', 'Deskripsi todo')}
					>
						<Copy class="size-3.5 text-muted-foreground" />
						<span>Salin Deskripsi</span>
					</button>
				{/if}
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg text-destructive hover:bg-destructive/10 transition-colors cursor-pointer text-left mt-0.5"
					onclick={() => {
						onDelete?.(t);
						close();
					}}
				>
					<Trash class="size-3.5" />
					<span>Hapus Todo</span>
				</button>
			</div>

		{:else if menuState.type === 'background'}
			<!-- Background Actions -->
			<div class="px-2 py-1 border-b border-border/60">
				<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">
					{projectName}
				</span>
			</div>

			<div class="py-1 border-b border-border/60">
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1.5 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left font-medium"
					onclick={() => {
						onFocusNewInput?.();
						close();
					}}
				>
					<Plus class="size-3.5 text-primary" />
					<span>Tulis Todo Baru</span>
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onOpenProjectPicker?.();
						close();
					}}
				>
					<FolderOpen class="size-3.5 text-muted-foreground" />
					<span>Pindah Space</span>
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onOpenCategoriesView?.();
						close();
					}}
				>
					<Tags class="size-3.5 text-muted-foreground" />
					<span>Kelola Kategori</span>
				</button>
			</div>

			<div class="py-1 border-b border-border/60">
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onToggleTheme?.();
						close();
					}}
				>
					{#if dark}
						<Sun class="size-3.5 text-amber-500" />
						<span>Mode Terang</span>
					{:else}
						<Moon class="size-3.5 text-blue-500" />
						<span>Mode Gelap</span>
					{/if}
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onLockApp?.();
						close();
					}}
				>
					<Lock class="size-3.5 text-muted-foreground" />
					<span>Kunci Aplikasi</span>
				</button>
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onOpenSecuritySettings?.();
						close();
					}}
				>
					<ShieldCheck class="size-3.5 text-primary" />
					<span>Pengaturan Kunci Otomatis</span>
				</button>
			</div>

			<div class="pt-1">
				<button
					type="button"
					class="flex w-full items-center gap-2 px-2 py-1 text-xs rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer text-left"
					onclick={() => {
						onOpenAbout?.();
						close();
					}}
				>
					<Info class="size-3.5" />
					<span>Tentang CheckMat</span>
				</button>
			</div>
		{/if}
	</div>
{/if}
