<script lang="ts">
	import {
		Plus,
		FolderOpen,
		Pencil,
		Trash,
		Lock,
		Sun,
		Moon,
		Search,
		X,
		EllipsisVertical,
		Check,
		CheckCircle2,
		RotateCcw,
		Archive,
		ArchiveRestore,
		GripVertical,
		Info
	} from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js';
	import type { Project, ProjectStatus } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		projects = [],
		dark = false,
		onPickProject,
		onOpenProjectDialog,
		onDeleteProject,
		onSetProjectStatus,
		onReorderProjects,
		onLockApp,
		onToggleTheme,
		onOpenAbout
	}: {
		projects: Project[];
		dark: boolean;
		onPickProject: (id: number) => void;
		onOpenProjectDialog: (mode: 'add' | 'edit', project?: Project) => void;
		onDeleteProject: (project: Project) => void;
		onSetProjectStatus?: (id: number, status: ProjectStatus) => Promise<void>;
		onReorderProjects?: (ids: number[]) => Promise<void>;
		onLockApp: () => void;
		onToggleTheme: () => void;
		onOpenAbout?: () => void;
	} = $props();

	let searchQuery = $state('');
	let statusFilter = $state<'all' | 'active' | 'completed' | 'archived'>('all');

	let dragProjectId = $state<number | null>(null);
	let overProjectIdx = $state<number | null>(null);
	let dragProjects = $state<Project[] | null>(null);

	const isReorderable = $derived(searchQuery.trim() === '' && statusFilter === 'all');

	const counts = $derived({
		all: projects.length,
		active: projects.filter((p) => (p.status ?? 'active') === 'active').length,
		completed: projects.filter((p) => p.status === 'completed').length,
		archived: projects.filter((p) => p.status === 'archived').length
	});

	const filteredProjects = $derived.by(() => {
		let list = projects;
		if (statusFilter !== 'all') {
			list = list.filter((p) => (p.status ?? 'active') === statusFilter);
		}
		if (searchQuery.trim() !== '') {
			const q = searchQuery.trim().toLowerCase();
			list = list.filter(
				(p) =>
					p.name.toLowerCase().includes(q) ||
					(p.description && p.description.toLowerCase().includes(q))
			);
		}
		return list;
	});

	const viewProjects = $derived(
		isReorderable ? (dragProjects ?? filteredProjects) : filteredProjects
	);

	function shiftProject(idx: number) {
		const cur = dragProjects ?? projects;
		const from = cur.findIndex((r) => r.id === dragProjectId);
		if (from === -1) return;
		const next = [...cur];
		const [m] = next.splice(from, 1);
		next.splice(idx > from ? idx - 1 : idx, 0, m);
		dragProjects = next;
	}

	function onDragStart(e: DragEvent, p: Project) {
		if (!isReorderable) return;
		dragProjectId = p.id;
		dragProjects = [...projects];
		overProjectIdx = projects.findIndex((r) => r.id === p.id);
		e.dataTransfer?.setData('text/plain', String(p.id));
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onRowDragOver(e: DragEvent, i: number) {
		if (!isReorderable) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const mid = rect.top + rect.height / 2;
		const insertIdx = e.clientY < mid ? i : i + 1;
		if (insertIdx !== overProjectIdx) {
			overProjectIdx = insertIdx;
			shiftProject(insertIdx);
		}
	}

	function onMainDragOver(e: DragEvent) {
		if (!isReorderable) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		const items = document.querySelectorAll('[data-project-item]');
		if (items.length === 0) return;
		const last = items[items.length - 1].getBoundingClientRect();
		if (e.clientY > last.bottom && overProjectIdx !== items.length) {
			overProjectIdx = items.length;
			shiftProject(items.length);
		}
	}

	async function onDrop(e: DragEvent) {
		if (!isReorderable) return;
		e.preventDefault();
		const final = dragProjects;
		dragProjectId = null;
		overProjectIdx = null;
		dragProjects = null;
		if (!final) return;
		const currentIds = projects.map((p) => p.id);
		const newIds = final.map((p) => p.id);
		if (newIds.every((id, idx) => id === currentIds[idx])) return;
		if (onReorderProjects) {
			await onReorderProjects(newIds);
		}
	}

	function onDragEnd() {
		dragProjectId = null;
		overProjectIdx = null;
		dragProjects = null;
	}

	const tabs: { k: 'all' | 'active' | 'completed' | 'archived'; label: string }[] = [
		{ k: 'all', label: 'Semua' },
		{ k: 'active', label: 'Aktif' },
		{ k: 'completed', label: 'Selesai' },
		{ k: 'archived', label: 'Arsip' }
	];
</script>

<div class="flex h-full flex-col p-3">
	<!-- Header -->
	<header class="flex items-center justify-between gap-2 pb-2.5">
		<div class="min-w-0">
			<h1 class="text-sm font-bold tracking-tight">Space</h1>
			<p class="text-[11px] text-muted-foreground">Pilih space untuk membuka daftar todo.</p>
		</div>
		<Button size="sm" onclick={() => onOpenProjectDialog('add')}>
			<Plus /> Space
		</Button>
	</header>

	<!-- Search & Filter Controls -->
	<div class="space-y-2 pb-2.5">
		<!-- Search Input -->
		<div class="relative flex items-center">
			<Search class="absolute left-2.5 size-3.5 text-muted-foreground pointer-events-none" />
			<Input
				bind:value={searchQuery}
				placeholder="Cari space..."
				class="h-8 pl-8 pr-7 text-xs"
			/>
			{#if searchQuery}
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<button
								{...props}
								type="button"
								onclick={() => (searchQuery = '')}
								class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
							>
								<X class="size-3.5" />
							</button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Hapus pencarian</Tooltip.Content>
				</Tooltip.Root>
			{/if}
		</div>

		<!-- Status Pills -->
		<div class="flex items-center gap-1 overflow-x-auto pb-0.5">
			{#each tabs as { k, label } (k)}
				{@const count = counts[k]}
				<button
					type="button"
					class={cn(
						'flex items-center gap-1.5 rounded-full px-2.5 py-1 text-[11px] font-semibold transition-colors cursor-pointer border',
						statusFilter === k
							? 'border-primary bg-primary text-primary-foreground shadow-xs'
							: 'border-border bg-card text-muted-foreground hover:border-foreground/30 hover:text-foreground'
					)}
					onclick={() => (statusFilter = k)}
				>
					<span>{label}</span>
					<span
						class={cn(
							'rounded-full px-1.5 py-0.2 text-[10px] font-bold',
							statusFilter === k
								? 'bg-primary-foreground/20 text-primary-foreground'
								: 'bg-muted text-muted-foreground'
						)}
					>
						{count}
					</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Space List -->
	<main
		class="flex-1 space-y-2 overflow-y-auto pr-0.5"
		ondragover={isReorderable ? onMainDragOver : undefined}
		ondrop={isReorderable ? onDrop : undefined}
	>
		{#if projects.length === 0}
			<div class="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground">
				Belum ada space. Buat space pertama untuk mulai mencatat todo.
			</div>
		{:else if filteredProjects.length === 0}
			<div class="rounded-lg border border-dashed p-4 text-center text-xs text-muted-foreground space-y-2">
				<p>Tidak ada space yang cocok.</p>
				<Button
					variant="outline"
					size="xs"
					onclick={() => {
						searchQuery = '';
						statusFilter = 'all';
					}}
				>
					Reset Pencarian & Filter
				</Button>
			</div>
		{/if}

		{#each viewProjects as project, idx (project.id)}
			{@const isCompleted = project.status === 'completed'}
			{@const isArchived = project.status === 'archived'}
			{@const isDragging = project.id === dragProjectId}
			<div
				data-project-item
				role="listitem"
				draggable={isReorderable}
				ondragstart={(e) => onDragStart(e, project)}
				ondragover={(e) => onRowDragOver(e, idx)}
				ondrop={onDrop}
				ondragend={onDragEnd}
				class={cn(
					'flex items-center gap-2 rounded-lg border bg-card p-2.5 transition-all duration-150 shadow-xs hover:border-foreground/20',
					isCompleted && 'bg-muted/30 border-emerald-500/30',
					isArchived && 'bg-muted/40 opacity-75 border-dashed',
					isDragging && 'opacity-40 border-primary ring-2 ring-primary/40 bg-accent/40 shadow-md'
				)}
			>
				{#if projects.length > 1}
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<div
									{...props}
									class={cn(
										'touch-none select-none p-0.5 -ml-0.5 shrink-0 transition-colors',
										isReorderable
											? 'cursor-grab active:cursor-grabbing text-muted-foreground/50 hover:text-foreground'
											: 'cursor-not-allowed text-muted-foreground/20'
									)}
								>
									<GripVertical class="size-4" />
								</div>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>
							{isReorderable
								? 'Geser posisi space (Drag & Drop)'
								: 'Urutan hanya dapat diatur saat menampilkan Semua status tanpa pencarian'}
						</Tooltip.Content>
					</Tooltip.Root>
				{/if}
				<button
					type="button"
					class="flex min-w-0 flex-1 items-start gap-2.5 text-left cursor-pointer"
					onclick={() => onPickProject(project.id)}
				>
					{#if isCompleted}
						<CheckCircle2 class="mt-0.5 size-5 shrink-0 text-emerald-500" />
					{:else if isArchived}
						<Archive class="mt-0.5 size-5 shrink-0 text-muted-foreground" />
					{:else}
						<FolderOpen class="mt-0.5 size-5 shrink-0 text-primary" />
					{/if}

					<span class="min-w-0 flex-1">
						<span class="flex items-center gap-1.5">
							<span
								class={cn(
									'truncate text-xs font-semibold',
									isCompleted && 'line-through text-muted-foreground',
									isArchived && 'text-muted-foreground'
								)}
							>
								{project.name}
							</span>
							{#if isCompleted}
								<span
									class="inline-flex items-center gap-1 rounded-full border border-emerald-200 bg-emerald-50 px-1.5 py-0.2 text-[10px] font-medium text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950/80 dark:text-emerald-400 shrink-0"
								>
									<Check class="size-2.5 stroke-[2.5]" /> Selesai
								</span>
							{:else if isArchived}
								<span
									class="inline-flex items-center gap-1 rounded-full border border-border bg-muted px-1.5 py-0.2 text-[10px] font-medium text-muted-foreground shrink-0"
								>
									<Archive class="size-2.5" /> Arsip
								</span>
							{/if}
						</span>
						<span class="mt-0.5 line-clamp-2 block text-xs text-muted-foreground">
							{project.description || 'Belum ada deskripsi'}
						</span>
					</span>
				</button>

				<!-- Dropdown Menu Aksi -->
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon-sm"
								class="text-muted-foreground hover:text-foreground shrink-0"
							>
								<EllipsisVertical class="size-4" />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end" class="min-w-40 w-40 rounded-lg p-1 shadow-lg border border-border">
						<DropdownMenu.Item onclick={() => onPickProject(project.id)} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
							<FolderOpen class="size-3.5 text-primary" /> Buka Space
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => onOpenProjectDialog('edit', project)} class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer">
							<Pencil class="size-3.5" /> Edit Space
						</DropdownMenu.Item>

						{#if onSetProjectStatus}
							<DropdownMenu.Separator />
							{#if !isCompleted}
								<DropdownMenu.Item
									onclick={() => onSetProjectStatus(project.id, 'completed')}
									class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-emerald-600 dark:text-emerald-400"
								>
									<CheckCircle2 class="size-3.5" /> Tandai Selesai
								</DropdownMenu.Item>
							{:else}
								<DropdownMenu.Item
									onclick={() => onSetProjectStatus(project.id, 'active')}
									class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
								>
									<RotateCcw class="size-3.5 text-blue-500" /> Aktifkan Kembali
								</DropdownMenu.Item>
							{/if}

							{#if !isArchived}
								<DropdownMenu.Item
									onclick={() => onSetProjectStatus(project.id, 'archived')}
									class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer text-muted-foreground"
								>
									<Archive class="size-3.5" /> Arsipkan
								</DropdownMenu.Item>
							{:else}
								<DropdownMenu.Item
									onclick={() => onSetProjectStatus(project.id, 'active')}
									class="py-1.5 px-2 text-xs rounded-md gap-2 cursor-pointer"
								>
									<ArchiveRestore class="size-3.5 text-blue-500" /> Buka dari Arsip
								</DropdownMenu.Item>
							{/if}
						{/if}

						<DropdownMenu.Separator />
						<DropdownMenu.Item
							disabled={projects.length <= 1}
							onclick={() => onDeleteProject(project)}
							class="py-1.5 px-2 text-xs rounded-md gap-2 text-destructive focus:bg-destructive/10 focus:text-destructive cursor-pointer"
						>
							<Trash class="size-3.5" /> Hapus Space
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</div>
		{/each}
	</main>

	<!-- Footer -->
	<footer class="flex items-center justify-between pt-2">
		{#if onOpenAbout}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button {...props} variant="ghost" size="icon-sm" class="text-muted-foreground hover:text-foreground" onclick={onOpenAbout}>
							<Info class="size-4" />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Tentang aplikasi</Tooltip.Content>
			</Tooltip.Root>
		{:else}
			<div></div>
		{/if}
		<div class="flex items-center gap-1.5">
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={onLockApp}>
						<Lock />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Kunci aplikasi</Tooltip.Content>
		</Tooltip.Root>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={onToggleTheme}>
						{#if dark}<Sun />{:else}<Moon />{/if}
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>{dark ? 'Mode terang' : 'Mode gelap'}</Tooltip.Content>
		</Tooltip.Root>
		</div>
	</footer>
</div>
