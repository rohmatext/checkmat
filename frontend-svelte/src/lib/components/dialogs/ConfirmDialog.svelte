<script lang="ts">
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.js';

	let {
		confirmState = $bindable<{
			msg: string;
			run: () => Promise<void>;
			variant?: 'destructive' | 'default';
			actionLabel?: string;
		} | null>(null)
	}: {
		confirmState: {
			msg: string;
			run: () => Promise<void>;
			variant?: 'destructive' | 'default';
			actionLabel?: string;
		} | null;
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

	const isDestructive = $derived(
		confirmState?.variant === 'destructive' ||
		(confirmState?.variant === undefined && /hapus/i.test(confirmState?.msg ?? ''))
	);
</script>

<AlertDialog.Root open={confirmState !== null} onOpenChange={(open) => { if (!open) confirmState = null; }}>
	<AlertDialog.Content class="max-w-xs gap-4 p-4 !z-[100]">
		<AlertDialog.Header class="gap-1 text-left">
			<AlertDialog.Title class="text-sm font-semibold">Konfirmasi</AlertDialog.Title>
			<AlertDialog.Description class="text-xs leading-relaxed break-words [overflow-wrap:anywhere] select-text">{confirmState?.msg ?? ''}</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="gap-1.5 flex-wrap">
			<AlertDialog.Cancel variant="outline" size="sm" disabled={busy}>Batal</AlertDialog.Cancel>
			<AlertDialog.Action
				size="sm"
				variant={isDestructive ? 'destructive' : 'default'}
				onclick={handleConfirm}
				disabled={busy}
			>
				{busy ? 'Memproses...' : (confirmState?.actionLabel ?? 'Lanjutkan')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<style>
	:global([data-slot="alert-dialog-overlay"]) {
		z-index: 99 !important;
	}
	:global([data-slot="alert-dialog-content"]) {
		z-index: 100 !important;
	}
</style>
