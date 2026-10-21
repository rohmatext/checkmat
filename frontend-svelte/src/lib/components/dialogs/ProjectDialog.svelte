<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { toast } from 'svelte-sonner';
	import type { Project } from '#lib/types.js';

	let {
		mode = $bindable<'add' | 'edit' | null>(null),
		project = null,
		onSaved
	}: {
		mode: 'add' | 'edit' | null;
		project: Project | null;
		onSaved: (id?: number) => Promise<void>;
	} = $props();

	let projectName = $state('');
	let projectDescription = $state('');
	let projectErr = $state('');

	$effect(() => {
		if (mode !== null) {
			projectName = mode === 'edit' ? (project?.name ?? '') : '';
			projectDescription = mode === 'edit' ? (project?.description ?? '') : '';
			projectErr = '';
		}
	});

	async function submitProject() {
		const name = projectName.trim();
		if (!name) return;
		try {
			if (mode === 'add') {
				const id = await invoke<number>('add_project', { name, description: projectDescription });
				await onSaved(id);
				toast.success('Space berhasil ditambahkan');
			} else if (project) {
				await invoke('update_project', {
					id: project.id,
					name,
					description: projectDescription
				});
				await onSaved(project.id);
				toast.success('Space berhasil diperbarui');
			}
			mode = null;
		} catch (e) {
			projectErr = String(e);
		}
	}
</script>

<Dialog.Root open={mode !== null} onOpenChange={(open) => { if (!open) mode = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title class="text-sm font-semibold">{mode === 'add' ? 'Tambah space' : 'Edit space'}</Dialog.Title>
			<Dialog.Description class="text-xs">
				{mode === 'add' ? 'Space baru akan dibuat dengan kategori Umum.' : 'Perbarui nama dan deskripsi space.'}
			</Dialog.Description>
		</Dialog.Header>
		<!-- svelte-ignore a11y_autofocus -->
		<Input
			bind:value={projectName}
			autofocus
			placeholder="Nama space"
			class="h-8 text-xs"
			onkeydown={(e) => e.key === 'Enter' && submitProject()}
		/>
		<textarea
			bind:value={projectDescription}
			placeholder="Deskripsi singkat (opsional)"
			rows="3"
			class="w-full resize-none rounded-md border border-input bg-transparent px-2.5 py-2 text-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30"
		></textarea>
		{#if projectErr}
			<p class="text-xs text-destructive">{projectErr}</p>
		{/if}
		<Dialog.Footer class="gap-1.5">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={submitProject}>Simpan</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
