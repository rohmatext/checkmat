<script lang="ts">
	import { FolderOpen, CheckCircle2, Archive, Pencil, CheckSquare, Clock, ListTodo, Trash2 } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import type { Project, ProjectStatus, Task } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		open = $bindable(false),
		project = null,
		tasks = [],
		onEditProject,
		onSetStatus
	}: {
		open: boolean;
		project: Project | null;
		tasks: Task[];
		onEditProject?: () => void;
		onSetStatus?: (id: number, status: ProjectStatus) => void;
	} = $props();

	// Calculate statistics for tasks in this project
	const projectTasks = $derived(
		project ? tasks.filter((t) => t.project_id === project.id) : []
	);
	const activeTasks = $derived(projectTasks.filter((t) => !t.deleted));
	const todoCount = $derived(
		activeTasks.filter((t) => !t.done && (!t.status || t.status === 'todo')).length
	);
	const inProgressCount = $derived(
		activeTasks.filter((t) => !t.done && t.status === 'inprogress').length
	);
	const doneCount = $derived(
		activeTasks.filter((t) => t.done || t.status === 'done').length
	);
	const trashCount = $derived(projectTasks.filter((t) => t.deleted).length);

	const progressPercent = $derived(
		activeTasks.length > 0 ? Math.round((doneCount / activeTasks.length) * 100) : 0
	);
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="w-[calc(100%-2rem)] max-w-sm gap-4 p-5 overflow-hidden">
		{#if project}
			<Dialog.Header class="gap-2 text-left pr-7">
				<!-- Icon Space + Badge Status -->
				<div class="flex items-center gap-1.5 flex-wrap">
					<div class="flex size-6 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
						{#if project.status === 'completed'}
							<CheckCircle2 class="size-3.5 text-emerald-500" />
						{:else if project.status === 'archived'}
							<Archive class="size-3.5 text-muted-foreground" />
						{:else}
							<FolderOpen class="size-3.5 text-primary" />
						{/if}
					</div>
					{#if project.status === 'completed'}
						<span class="rounded-full bg-emerald-50 px-2 py-0.5 text-[10px] font-medium text-emerald-700 dark:bg-emerald-950/80 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800 shrink-0">
							Selesai
						</span>
					{:else if project.status === 'archived'}
						<span class="rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground border border-border shrink-0">
							Arsip
						</span>
					{:else}
						<span class="rounded-full bg-blue-50 px-2 py-0.5 text-[10px] font-medium text-blue-700 dark:bg-blue-950/80 dark:text-blue-400 border border-blue-200 dark:border-blue-800 shrink-0">
							Aktif
						</span>
					{/if}
				</div>

				<!-- Judul Space dengan text wrap / break words -->
				<Dialog.Title class="text-base font-semibold leading-snug break-words [overflow-wrap:anywhere] select-text">
					{project.name}
				</Dialog.Title>

				<!-- Deskripsi Space dengan text wrap & scroll jika panjang -->
				{#if project.description}
					<div class="mt-0.5 max-h-36 overflow-y-auto rounded-lg border bg-muted/30 p-2.5 text-xs leading-relaxed text-foreground select-text whitespace-pre-wrap break-words [overflow-wrap:anywhere]">
						{project.description}
					</div>
				{:else}
					<Dialog.Description class="text-xs text-muted-foreground mt-0.5">
						Tidak ada deskripsi untuk space ini.
					</Dialog.Description>
				{/if}
			</Dialog.Header>

			<!-- Progress Bar -->
			<div class="space-y-1.5 rounded-lg border border-border bg-muted/30 p-3">
				<div class="flex items-center justify-between text-xs font-medium">
					<span class="text-muted-foreground">Progres Penyelesaian</span>
					<span class="font-semibold text-foreground">{progressPercent}%</span>
				</div>
				<div class="h-2 w-full overflow-hidden rounded-full bg-muted">
					<div
						class="h-full bg-primary transition-all duration-300 rounded-full"
						style="width: {progressPercent}%"
					></div>
				</div>
			</div>

			<!-- Stats Grid -->
			<div class="grid grid-cols-2 gap-2 text-xs">
				<div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2.5 min-w-0">
					<div class="flex size-7 shrink-0 items-center justify-center rounded-md bg-blue-500/10 text-blue-600 dark:text-blue-400">
						<ListTodo class="size-3.5" />
					</div>
					<div class="min-w-0">
						<p class="text-[10px] font-medium text-muted-foreground">Todo</p>
						<p class="text-sm font-semibold">{todoCount}</p>
					</div>
				</div>

				<div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2.5 min-w-0">
					<div class="flex size-7 shrink-0 items-center justify-center rounded-md bg-amber-500/10 text-amber-600 dark:text-amber-400">
						<Clock class="size-3.5" />
					</div>
					<div class="min-w-0">
						<p class="text-[10px] font-medium text-muted-foreground">In Progress</p>
						<p class="text-sm font-semibold">{inProgressCount}</p>
					</div>
				</div>

				<div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2.5 min-w-0">
					<div class="flex size-7 shrink-0 items-center justify-center rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
						<CheckSquare class="size-3.5" />
					</div>
					<div class="min-w-0">
						<p class="text-[10px] font-medium text-muted-foreground">Selesai</p>
						<p class="text-sm font-semibold">{doneCount}</p>
					</div>
				</div>

				<div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2.5 min-w-0">
					<div class="flex size-7 shrink-0 items-center justify-center rounded-md bg-rose-500/10 text-rose-600 dark:text-rose-400">
						<Trash2 class="size-3.5" />
					</div>
					<div class="min-w-0">
						<p class="text-[10px] font-medium text-muted-foreground">Sampah</p>
						<p class="text-sm font-semibold">{trashCount}</p>
					</div>
				</div>
			</div>

			<!-- Status Quick Switch (Optional) -->
			{#if onSetStatus}
				<div class="flex items-center justify-between gap-1.5 pt-1 border-t border-border/60 flex-wrap">
					<span class="text-[11px] text-muted-foreground">Ubah status:</span>
					<div class="flex gap-1">
						<button
							type="button"
							onclick={() => project && onSetStatus?.(project.id, 'active')}
							class={cn(
								'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
								project.status === 'active' || !project.status
									? 'bg-primary text-primary-foreground font-semibold'
									: 'bg-muted hover:bg-muted/80 text-muted-foreground'
							)}
						>
							Aktif
						</button>
						<button
							type="button"
							onclick={() => project && onSetStatus?.(project.id, 'completed')}
							class={cn(
								'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
								project.status === 'completed'
									? 'bg-emerald-600 text-white font-semibold'
									: 'bg-muted hover:bg-muted/80 text-muted-foreground'
							)}
						>
							Selesai
						</button>
						<button
							type="button"
							onclick={() => project && onSetStatus?.(project.id, 'archived')}
							class={cn(
								'rounded px-2 py-0.5 text-[10px] font-medium transition-colors cursor-pointer',
								project.status === 'archived'
									? 'bg-zinc-700 text-white dark:bg-zinc-600 font-semibold'
									: 'bg-muted hover:bg-muted/80 text-muted-foreground'
							)}
						>
							Arsip
						</button>
					</div>
				</div>
			{/if}

			<Dialog.Footer class="gap-1.5 pt-2 sm:justify-between">
				<Button
					variant="outline"
					size="sm"
					class="gap-1.5 text-xs"
					onclick={() => {
						open = false;
						onEditProject?.();
					}}
				>
					<Pencil class="size-3.5" />
					Edit Space
				</Button>
				<Dialog.Close class={buttonVariants({ variant: 'secondary', size: 'sm' })}>
					Tutup
				</Dialog.Close>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
