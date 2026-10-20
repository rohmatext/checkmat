<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Lock } from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { toast } from 'svelte-sonner';

	let {
		locked = $bindable(true),
		pinSet = $bindable(false)
	} = $props();

	let lockStage = $state<'verify' | 'setup' | 'confirm'>('verify');
	let lockInput = $state('');
	let lockConfirmInput = $state('');
	let lockError = $state('');
	let lockBusy = $state(false);

	$effect(() => {
		if (locked) {
			lockStage = pinSet ? 'verify' : 'setup';
			lockInput = '';
			lockConfirmInput = '';
			lockError = '';
		}
	});

	async function submitLock() {
		if (lockBusy) return;
		lockError = '';
		if (lockStage === 'setup') {
			if (lockInput.length < 4) {
				lockError = 'PIN minimal 4 digit.';
				return;
			}
			lockConfirmInput = '';
			lockStage = 'confirm';
			return;
		}
		if (lockStage === 'confirm') {
			if (lockInput !== lockConfirmInput) {
				lockError = 'PIN tidak sama, coba lagi.';
				lockConfirmInput = '';
				return;
			}
			lockBusy = true;
			try {
				await invoke('set_pin', { pin: lockInput });
				pinSet = true;
				locked = false;
				toast.success('PIN berhasil dibuat');
			} catch (e) {
				lockError = String(e);
			} finally {
				lockBusy = false;
			}
			return;
		}
		// verify
		lockBusy = true;
		try {
			const ok = await invoke<boolean>('verify_pin', { pin: lockInput });
			if (ok) {
				locked = false;
			} else {
				lockError = 'PIN salah.';
				lockInput = '';
			}
		} catch (e) {
			lockError = String(e);
		} finally {
			lockBusy = false;
		}
	}

	async function resetPin() {
		try {
			await invoke('clear_pin');
			pinSet = false;
			lockStage = 'setup';
			lockInput = '';
			lockConfirmInput = '';
			lockError = '';
			toast.success('PIN direset. Buat PIN baru.');
		} catch (e) {
			toast.error(String(e));
		}
	}
</script>

{#if locked}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/95 p-6 backdrop-blur">
		<div class="flex w-full max-w-xs flex-col items-center gap-4">
			<div class="flex size-14 items-center justify-center rounded-full border bg-card">
				<Lock class="size-6 text-primary" />
			</div>
			<div class="text-center">
				<h2 class="text-lg font-bold">
					{lockStage === 'verify' ? 'Aplikasi Terkunci' : lockStage === 'setup' ? 'Buat PIN' : 'Konfirmasi PIN'}
				</h2>
				<p class="mt-1 text-xs text-muted-foreground">
					{lockStage === 'verify'
						? 'Masukkan PIN untuk membuka aplikasi.'
						: lockStage === 'setup'
							? 'Buat minimal 4 digit PIN baru.'
							: 'Ketik ulang PIN untuk memastikan.'}
				</p>
			</div>
			<div class="w-full space-y-2">
				{#if lockStage === 'confirm'}
					<Input
						type="password"
						bind:value={lockConfirmInput}
						autofocus
						placeholder="Konfirmasi PIN"
						class="text-center text-sm"
						onkeydown={(e) => e.key === 'Enter' && submitLock()}
					/>
				{:else}
					<Input
						type="password"
						bind:value={lockInput}
						autofocus
						placeholder="Masukkan PIN"
						class="text-center text-sm"
						onkeydown={(e) => e.key === 'Enter' && submitLock()}
					/>
				{/if}
				{#if lockError}
					<p class="text-center text-xs text-destructive">{lockError}</p>
				{/if}
			</div>
			<div class="flex w-full flex-col gap-2">
				<Button class="w-full" onclick={submitLock} disabled={lockBusy}>
					{lockBusy
						? 'Memproses...'
						: lockStage === 'verify'
							? 'Buka'
							: lockStage === 'setup'
								? 'Lanjut'
								: 'Simpan PIN'}
				</Button>
				{#if lockStage === 'confirm'}
					<Button
						variant="ghost"
						size="sm"
						onclick={() => {
							lockStage = 'setup';
							lockConfirmInput = '';
							lockError = '';
						}}
					>
						Kembali
					</Button>
				{:else if pinSet}
					<button
						type="button"
						class="text-xs text-muted-foreground underline hover:text-foreground"
						onclick={resetPin}
					>
						Reset PIN
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
