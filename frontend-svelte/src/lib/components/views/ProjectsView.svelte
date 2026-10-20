<script lang="ts">
	import { Plus, FolderOpen, Pencil, Trash, Lock, Sun, Moon } from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import type { Project } from '#lib/types.js';

	let {
		projects = [],
		dark = false,
		onPickProject,
		onOpenProjectDialog,
		onDeleteProject,
		onLockApp,
		onToggleTheme
	}: {
		projects: Project[];
		dark: boolean;
		onPickProject: (id: number) => void;
		onOpenProjectDialog: (mode: 'add' | 'edit', project?: Project) => void;
		onDeleteProject: (project: Project) => void;
		onLockApp: () => void;
		onToggleTheme: () => void;
	} = $props();
</script>

<div class="flex h-full flex-col p-3">
	<header class="flex items-center justify-between gap-2 pb-3">
		<div class="min-w-0">
			<h1 class="text-lg font-bold">Project</h1>
			<p class="text-xs text-muted-foreground">Pilih project untuk membuka daftar tugas.</p>
		</div>
		<Button size="sm" onclick={() => onOpenProjectDialog('add')}><Plus /> Project</Button>
	</header>

	<main class="flex-1 space-y-2 overflow-y-auto">
		{#if projects.length === 0}
			<div class="rounded-lg border border-dashed p-6 text-center text-sm text-muted-foreground">
				Belum ada project. Buat project pertama untuk mulai mencatat todo.
			</div>
		{/if}
		{#each projects as project (project.id)}
			<div class="flex items-center gap-2 rounded-lg border bg-card p-3">
				<button
					type="button"
					class="flex min-w-0 flex-1 items-start gap-3 text-left cursor-pointer"
					onclick={() => onPickProject(project.id)}
				>
					<FolderOpen class="mt-0.5 size-5 shrink-0 text-primary" />
					<span class="min-w-0 flex-1">
						<span class="block truncate text-sm font-semibold">{project.name}</span>
						<span class="mt-0.5 line-clamp-2 block text-xs text-muted-foreground">
							{project.description || 'Belum ada deskripsi'}
						</span>
					</span>
				</button>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="outline"
								size="icon-sm"
								title="Edit project"
								onclick={() => onOpenProjectDialog('edit', project)}
							>
								<Pencil />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Edit project</Tooltip.Content>
				</Tooltip.Root>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="outline"
								size="icon-sm"
								disabled={projects.length <= 1}
								title="Hapus project"
								class="hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
								onclick={() => onDeleteProject(project)}
							>
								<Trash />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Hapus project</Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/each}
	</main>

	<footer class="flex justify-end gap-1.5 pt-2">
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" title="Kunci aplikasi" onclick={onLockApp}>
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
	</footer>
</div>
