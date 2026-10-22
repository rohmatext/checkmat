<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { FolderOpen } from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { EXPORT_DIR_KEY } from '#lib/types.js';

	let {
		open = $bindable(false),
		exportDirectory = $bindable('')
	} = $props();

	let exportFileName = $state('');
	let fileActionError = $state('');

	function timestampFileName() {
		const d = new Date();
		const part = (v: number) => String(v).padStart(2, '0');
		return `${part(d.getFullYear() % 100)}${part(d.getMonth() + 1)}${part(d.getDate())}${part(d.getHours())}${part(d.getMinutes())}${part(d.getSeconds())}_checkmat`;
	}

	$effect(() => {
		if (open) {
			exportFileName = timestampFileName();
			fileActionError = '';
		}
	});

	async function chooseExportDirectory() {
		try {
			const selected = await openDialog({
				directory: true,
				multiple: false,
				title: 'Pilih folder untuk menyimpan CSV'
			});
			if (typeof selected === 'string') {
				exportDirectory = selected;
				localStorage.setItem(EXPORT_DIR_KEY, selected);
			}
		} catch (e) {
			fileActionError = String(e);
		}
	}

	async function exportCsv() {
		const name = exportFileName.trim().replace(/[<>:"/\\|?*]/g, '_');
		if (!exportDirectory || !name) {
			fileActionError = 'Pilih folder dan isi nama file.';
			return;
		}
		try {
			const path = `${exportDirectory.replace(/[\\/]+$/, '')}\\${name}.csv`;
			const savedPath = await invoke<string>('export_csv', { path });
			open = false;
			toast.success(`CSV disimpan ke ${savedPath}`);
		} catch (e) {
			fileActionError = String(e);
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-sm gap-4 p-4">
		<Dialog.Header class="gap-1">
			<Dialog.Title class="text-sm font-semibold">Export CSV</Dialog.Title>
			<Dialog.Description class="text-xs">
				Simpan seluruh todo ke file CSV beserta status dan waktunya.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-3">
			<div class="space-y-1">
				<label for="export-dir" class="text-xs font-medium text-muted-foreground">Folder tujuan</label>
				<div class="flex gap-1.5">
					<Input
						id="export-dir"
						value={exportDirectory}
						readonly
						placeholder="Pilih folder..."
						class="h-8 flex-1 text-xs"
					/>
					<Button variant="outline" size="sm" onclick={chooseExportDirectory} class="h-8 gap-1.5 px-2.5 text-xs">
						<FolderOpen class="size-3.5" />
						Pilih
					</Button>
				</div>
			</div>
			<div class="space-y-1">
				<label for="export-file" class="text-xs font-medium text-muted-foreground">Nama file</label>
				<div class="flex items-center gap-1.5">
					<Input
						id="export-file"
						bind:value={exportFileName}
						placeholder="nama_file"
						class="h-8 flex-1 text-xs"
						onkeydown={(e) => e.key === 'Enter' && exportCsv()}
					/>
					<span class="text-xs text-muted-foreground">.csv</span>
				</div>
			</div>
			{#if fileActionError}
				<p class="text-xs text-destructive">{fileActionError}</p>
			{/if}
		</div>
		<Dialog.Footer class="gap-1.5 flex-wrap">
			<Dialog.Close class={buttonVariants({ variant: 'outline', size: 'sm' })}>Batal</Dialog.Close>
			<Button size="sm" onclick={exportCsv}>Export</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
