<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { Upload } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { toast } from 'svelte-sonner';

	let {
		open = $bindable(false),
		onSuccess
	}: {
		open: boolean;
		onSuccess: () => Promise<void>;
	} = $props();

	let importProcessed = $state(0);
	let importTotal = $state(0);
	let importBusy = $state(false);
	let fileActionError = $state('');

	$effect(() => {
		if (open) {
			fileActionError = '';
			importProcessed = 0;
			importTotal = 0;
			importBusy = false;
		}
	});

	async function saveCsvTemplate() {
		try {
			const selected = await openDialog({
				directory: true,
				multiple: false,
				title: 'Pilih folder untuk menyimpan template CSV'
			});
			if (typeof selected !== 'string') return;
			const path = `${selected.replace(/[\\/]+$/, '')}\\template_taskmini.csv`;
			const savedPath = await invoke<string>('write_csv_template', { path });
			toast.success(`Template disimpan ke ${savedPath}`);
		} catch (e) {
			fileActionError = String(e);
		}
	}

	async function importCsv() {
		fileActionError = '';
		try {
			const selected = await openDialog({
				multiple: false,
				title: 'Pilih file CSV untuk diimport',
				filters: [{ name: 'CSV', extensions: ['csv'] }]
			});
			if (typeof selected !== 'string') return;
			importProcessed = 0;
			importTotal = 0;
			importBusy = true;
			let unlisten: (() => void) | undefined;
			try {
				unlisten = await listen<{ processed: number; total: number }>(
					'import-progress',
					(event) => {
						importProcessed = event.payload.processed;
						importTotal = event.payload.total;
					}
				);
				const count = await invoke<number>('import_csv', { path: selected });
				importProcessed = count;
				importTotal = count;
				await onSuccess();
				open = false;
				toast.success(`${count} todo berhasil diimport`);
			} catch (e) {
				fileActionError = String(e);
				toast.error(`Import gagal: ${String(e)}`);
			} finally {
				unlisten?.();
				importBusy = false;
			}
		} catch (e) {
			fileActionError = String(e);
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-xs gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title>Import CSV</Dialog.Title>
			<Dialog.Description class="text-xs">
				Format: id,text,project,category,done,deleted,created_at,done_at,deleted_at. Kolom text wajib.
			</Dialog.Description>
		</Dialog.Header>
		{#if importBusy}
			<div class="space-y-2 py-2">
				<div class="flex justify-between text-xs text-muted-foreground">
					<span>Mengimport todo...</span>
					<span>{importProcessed} / {importTotal || '...'}</span>
				</div>
				<progress
					class="h-2 w-full overflow-hidden rounded-full accent-primary"
					value={importProcessed}
					max={Math.max(importTotal, 1)}
				></progress>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				<Button onclick={importCsv}><Upload /> Pilih file CSV</Button>
				<Button variant="outline" onclick={saveCsvTemplate}>Simpan template CSV</Button>
			</div>
		{/if}
		{#if fileActionError}
			<p class="text-xs text-destructive">{fileActionError}</p>
		{/if}
		<Dialog.Footer>
			<Dialog.Close class={buttonVariants({ variant: 'ghost', size: 'sm' })}>Tutup</Dialog.Close>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
