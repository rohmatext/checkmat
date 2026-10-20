<script lang="ts">
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';

	let {
		confirmState = $bindable<{ msg: string; run: () => Promise<void> } | null>(null)
	}: {
		confirmState: { msg: string; run: () => Promise<void> } | null;
	} = $props();

	let busy = $state(false);

	async function handleConfirm() {
		if (!confirmState || busy) return;
		busy = true;
		try {
			await confirmState.run();
		} finally {
			busy = false;
			confirmState = null;
		}
	}
</script>

<AlertDialog.Root open={confirmState !== null} onOpenChange={(open) => { if (!open) confirmState = null; }}>
	<AlertDialog.Content class="max-w-xs gap-4 p-4">
		<AlertDialog.Header class="gap-1">
			<AlertDialog.Title>Konfirmasi</AlertDialog.Title>
			<AlertDialog.Description class="text-xs">{confirmState?.msg ?? ''}</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="gap-1.5">
			<AlertDialog.Cancel variant="outline" size="sm" disabled={busy}>Batal</AlertDialog.Cancel>
			<AlertDialog.Action
				size="sm"
				onclick={handleConfirm}
				disabled={busy}
				class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
			>
				{busy ? 'Memproses...' : 'Lanjutkan'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
