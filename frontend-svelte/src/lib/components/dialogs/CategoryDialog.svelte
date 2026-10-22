<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { toast } from 'svelte-sonner';
	import { PALETTE, TONE, getSolidCls, type Category } from '#lib/types.js';
	import { cn } from '#lib/utils';

	let {
		mode = $bindable<'add' | 'rename' | null>(null),
		category = null,
		projectId = null,
		onSaved
	}: {
		mode: 'add' | 'rename' | null;
		category: Category | null;
		projectId: number | null;
		onSaved: (id?: number) => Promise<void>;
	} = $props();

	let catName = $state('');
	let catColor = $state<string>(PALETTE[0]);
	let catErr = $state('');

	$effect(() => {
		if (mode !== null) {
			catName = mode === 'rename' ? (category?.name ?? '') : '';
			const c = mode === 'rename' ? category?.color : '';
			catColor = c && TONE[c] ? c : PALETTE[0];
			catErr = '';
		}
	});

	async function submitCat() {
		const name = catName.trim();
		if (!name) return;
		if (name.length > 50) {
			catErr = 'Nama kategori maksimal 50 karakter';
			return;
		}
		try {
			if (mode === 'add') {
				if (projectId === null) return;
				const id = await invoke<number>('add_category', {
					name,
					color: catColor,
					projectId
				});
				await onSaved(id);
				toast.success(`Kategori "${name}" berhasil ditambahkan`);
			} else if (category) {
				let changed = false;
				if (name !== category.name) {
					await invoke('rename_category', { id: category.id, name, projectId });
					changed = true;
				}
				if (catColor !== category.color) {
					await invoke('set_category_color', { id: category.id, color: catColor });
					changed = true;
				}
				await onSaved(category.id);
				if (changed) toast.success(`Kategori "${name}" berhasil diperbarui`);
			}
			mode = null;
		} catch (e) {
			catErr = String(e);
		}
	}
</script>

<Dialog.Root open={mode !== null} onOpenChange={(open) => { if (!open) mode = null; }}>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title class="text-sm font-semibold">{mode === 'add' ? 'Tambah kategori' : 'Ganti nama kategori'}</Dialog.Title>
			<Dialog.Description class="text-xs">
				{mode === 'add' ? 'Kategori baru langsung dipakai.' : `Nama sekarang: ${category?.name ?? ''}`}
			</Dialog.Description>
		</Dialog.Header>
		<!-- svelte-ignore a11y_autofocus -->
		<div class="space-y-1">
			<Input
				bind:value={catName}
				autofocus
				maxlength={50}
				placeholder="Nama kategori"
				class="h-8 text-xs"
				onkeydown={(e) => e.key === 'Enter' && submitCat()}
			/>
			<div class="flex justify-between items-center text-[10px] text-muted-foreground px-0.5">
				<span>{catErr ? '' : 'Maksimal 50 karakter'}</span>
				<span class={catName.length >= 50 ? 'text-destructive font-medium' : ''}>{catName.length}/50</span>
			</div>
		</div>
		<div class="flex flex-wrap gap-1.5">
			{#each PALETTE as c}
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<button
								{...props}
								type="button"
								onclick={() => (catColor = c)}
								class={cn(
									'size-5 cursor-pointer rounded-full border-2 transition-transform',
									getSolidCls(c),
									catColor === c ? 'scale-110 border-foreground' : 'border-transparent'
								)}
							></button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>{c}</Tooltip.Content>
				</Tooltip.Root>
			{/each}
		</div>
		{#if catErr}
			<p class="text-xs text-destructive">{catErr}</p>
		{/if}
		<Dialog.Footer class="gap-1.5">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={submitCat}>Simpan</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
