<script lang="ts">
	import { FolderOpen, Search, X, Check, Archive, CheckCircle2 } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import type { Project } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		open = $bindable(false),
		projects = [],
		activeProject = null,
		onSelect
	}: {
		open: boolean;
		projects: Project[];
		activeProject: number | null;
		onSelect: (id: number) => void;
	} = $props();

	let searchQuery = $state('');

	$effect(() => {
		if (open) {
			searchQuery = '';
		}
	});

	const filteredProjects = $derived(
		searchQuery.trim() === ''
			? projects
			: projects.filter(
					(p) =>
						p.name.toLowerCase().includes(searchQuery.trim().toLowerCase()) ||
						(p.description &&
							p.description.toLowerCase().includes(searchQuery.trim().toLowerCase()))
				)
	);
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-xs gap-3 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title class="text-sm font-semibold">Pilih space</Dialog.Title>
			<Dialog.Description class="text-xs">Beralih ke daftar tugas space lain.</Dialog.Description>
		</Dialog.Header>

		{#if projects.length > 0}
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
									<X class="size-3" />
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Hapus pencarian</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			</div>
		{/if}

		<div class="flex max-h-72 flex-col gap-1.5 overflow-y-auto pr-0.5">
			{#each filteredProjects as project (project.id)}
				{@const isCompleted = project.status === 'completed'}
				{@const isArchived = project.status === 'archived'}
				<Button
					variant={activeProject === project.id ? 'secondary' : 'outline'}
					size="sm"
					class={cn(
						'h-auto min-h-9 justify-start gap-2 py-1.5 px-2.5 text-left',
						isCompleted && 'opacity-85',
						isArchived && 'opacity-70 border-dashed'
					)}
					onclick={() => onSelect(project.id)}
				>
					{#if isCompleted}
						<CheckCircle2 class="size-4 shrink-0 text-emerald-500" />
					{:else if isArchived}
						<Archive class="size-4 shrink-0 text-muted-foreground" />
					{:else}
						<FolderOpen class="size-4 shrink-0 text-primary" />
					{/if}
					<span class="flex min-w-0 flex-1 items-center justify-between gap-1.5">
						<span class={cn('truncate min-w-0 flex-1', isCompleted && 'line-through text-muted-foreground')}>
							{project.name}
						</span>
						{#if isCompleted}
							<span
								class="rounded-full bg-emerald-50 px-1.5 py-0.2 text-[9px] font-medium text-emerald-700 dark:bg-emerald-950/80 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800 shrink-0"
							>
								Selesai
							</span>
						{:else if isArchived}
							<span
								class="rounded-full bg-muted px-1.5 py-0.2 text-[9px] font-medium text-muted-foreground border border-border shrink-0"
							>
								Arsip
							</span>
						{/if}
					</span>
				</Button>
			{/each}

			{#if filteredProjects.length === 0}
				<p class="py-3 text-center text-xs text-muted-foreground">
					{projects.length === 0 ? 'Belum ada space.' : 'Space tidak ditemukan.'}
				</p>
			{/if}
		</div>

		<Dialog.Footer class="gap-1.5 flex-wrap">
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Tutup</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
