<script lang="ts">
	import { FolderOpen } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import type { Project } from '#lib/types.js';

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
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Pilih project</Dialog.Title>
			<Dialog.Description class="text-xs">Beralih ke daftar tugas project lain.</Dialog.Description>
		</Dialog.Header>
		<div class="flex max-h-72 flex-col gap-1.5 overflow-y-auto">
			{#each projects as project (project.id)}
				<Button
					variant={activeProject === project.id ? 'secondary' : 'outline'}
					size="sm"
					class="justify-start gap-2"
					onclick={() => onSelect(project.id)}
				>
					<FolderOpen class="size-4 shrink-0" />
					<span class="truncate">{project.name}</span>
				</Button>
			{/each}
			{#if projects.length === 0}
				<p class="py-3 text-center text-xs text-muted-foreground">Belum ada project.</p>
			{/if}
		</div>
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Tutup</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
