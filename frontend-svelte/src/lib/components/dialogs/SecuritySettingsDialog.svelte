<script lang="ts">
	import {
		Shield,
		ShieldCheck,
		ShieldAlert,
		Lock,
		KeyRound,
		Clock,
		Check,
		AlertCircle,
		Sparkles
	} from '@lucide/svelte';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { AUTOLOCK_OPTIONS, AUTOLOCK_KEY } from '#lib/types.js';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import { cn } from '#lib/utils.js';

	let {
		open = $bindable(false),
		autoLockDelay = $bindable('never'),
		pinSet = $bindable(false),
		onLockNow
	}: {
		open: boolean;
		autoLockDelay: string;
		pinSet: boolean;
		onLockNow?: () => void;
	} = $props();

	// Modes: 'main' | 'set_pin' | 'change_pin' | 'clear_pin'
	let mode = $state<'main' | 'set_pin' | 'change_pin' | 'clear_pin'>('main');

	// Form states
	let oldPin = $state('');
	let newPin = $state('');
	let confirmPin = $state('');
	let errorMsg = $state('');
	let isBusy = $state(false);

	$effect(() => {
		if (open) {
			mode = 'main';
			oldPin = '';
			newPin = '';
			confirmPin = '';
			errorMsg = '';
		}
	});

	function handleSelectDelay(val: string) {
		autoLockDelay = val;
		localStorage.setItem(AUTOLOCK_KEY, val);
		const opt = AUTOLOCK_OPTIONS.find((o) => o.value === val);
		toast.success(`Kunci otomatis diatur: ${opt?.label ?? val}`);
	}

	async function handleSetPin() {
		errorMsg = '';
		if (newPin.trim().length < 4) {
			errorMsg = 'PIN minimal 4 digit.';
			return;
		}
		if (newPin !== confirmPin) {
			errorMsg = 'Konfirmasi PIN tidak cocok.';
			return;
		}
		isBusy = true;
		try {
			await invoke('set_pin', { pin: newPin.trim() });
			pinSet = true;
			mode = 'main';
			newPin = '';
			confirmPin = '';
			toast.success('PIN berhasil dibuat');
		} catch (e) {
			errorMsg = String(e);
		} finally {
			isBusy = false;
		}
	}

	async function handleChangePin() {
		errorMsg = '';
		if (!oldPin) {
			errorMsg = 'Masukkan PIN lama Anda.';
			return;
		}
		if (newPin.trim().length < 4) {
			errorMsg = 'PIN baru minimal 4 digit.';
			return;
		}
		if (newPin !== confirmPin) {
			errorMsg = 'Konfirmasi PIN baru tidak cocok.';
			return;
		}
		isBusy = true;
		try {
			const valid = await invoke<boolean>('verify_pin', { pin: oldPin.trim() });
			if (!valid) {
				errorMsg = 'PIN lama salah.';
				isBusy = false;
				return;
			}
			await invoke('set_pin', { pin: newPin.trim() });
			pinSet = true;
			mode = 'main';
			oldPin = '';
			newPin = '';
			confirmPin = '';
			toast.success('PIN berhasil diperbarui');
		} catch (e) {
			errorMsg = String(e);
		} finally {
			isBusy = false;
		}
	}

	async function handleClearPin() {
		errorMsg = '';
		if (!oldPin) {
			errorMsg = 'Masukkan PIN untuk konfirmasi penghapusan.';
			return;
		}
		isBusy = true;
		try {
			const valid = await invoke<boolean>('verify_pin', { pin: oldPin.trim() });
			if (!valid) {
				errorMsg = 'PIN salah.';
				isBusy = false;
				return;
			}
			await invoke('clear_pin');
			pinSet = false;
			mode = 'main';
			oldPin = '';
			toast.success('PIN berhasil dihapus. Aplikasi tidak lagi diproteksi PIN.');
		} catch (e) {
			errorMsg = String(e);
		} finally {
			isBusy = false;
		}
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-md p-0 overflow-hidden gap-0 rounded-2xl border-border/80 shadow-2xl">
		<!-- Header -->
		<div class="p-4 border-b border-border/70 bg-gradient-to-r from-primary/10 via-background to-muted/20">
			<div class="flex items-center gap-3">
				<div class="size-10 rounded-xl bg-primary/15 border border-primary/25 flex items-center justify-center text-primary shrink-0 shadow-xs">
					{#if pinSet}
						<ShieldCheck class="size-5" />
					{:else}
						<ShieldAlert class="size-5 text-amber-500" />
					{/if}
				</div>
				<div class="min-w-0 flex-1">
					<Dialog.Title class="text-sm font-bold tracking-tight">
						Keamanan & Kunci Otomatis
					</Dialog.Title>
					<Dialog.Description class="text-xs text-muted-foreground truncate">
						Pengaturan proteksi PIN dan penguncian saat minimize
					</Dialog.Description>
				</div>
			</div>
		</div>

		<!-- Body -->
		<div class="p-4 space-y-4 max-h-[70vh] overflow-y-auto">
			{#if mode === 'main'}
				<!-- PIN Status Card -->
				<div class="rounded-xl border border-border/70 bg-card p-3.5 space-y-3">
					<div class="flex items-center justify-between">
						<span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">
							Status Proteksi PIN
						</span>
						{#if pinSet}
							<Badge variant="outline" class="text-[10px] gap-1 px-2 py-0.5 border-emerald-500/30 text-emerald-600 dark:text-emerald-400 bg-emerald-500/10">
								<Check class="size-3" /> PIN Aktif
							</Badge>
						{:else}
							<Badge variant="outline" class="text-[10px] gap-1 px-2 py-0.5 border-amber-500/30 text-amber-600 dark:text-amber-400 bg-amber-500/10">
								<AlertCircle class="size-3" /> Belum Diatur
							</Badge>
						{/if}
					</div>

					<p class="text-xs text-muted-foreground leading-relaxed">
						{#if pinSet}
							Aplikasi dilindungi PIN. Kunci otomatis hanya akan berfungsi jika PIN aktif.
						{:else}
							Belum ada PIN yang dibuat. Buat PIN terlebih dahulu agar aplikasi dapat dikunci saat diminimize.
						{/if}
					</p>

					<div class="flex items-center gap-2 pt-1">
						{#if pinSet}
							<Button
								variant="outline"
								size="sm"
								class="flex-1 text-xs h-8 cursor-pointer gap-1.5"
								onclick={() => {
									mode = 'change_pin';
									errorMsg = '';
								}}
							>
								<KeyRound class="size-3.5 text-muted-foreground" />
								Ganti PIN
							</Button>
							<Button
								variant="ghost"
								size="sm"
								class="text-xs h-8 text-destructive hover:bg-destructive/10 cursor-pointer"
								onclick={() => {
									mode = 'clear_pin';
									errorMsg = '';
								}}
							>
								Hapus PIN
							</Button>
						{:else}
							<Button
								size="sm"
								class="w-full text-xs h-8 cursor-pointer gap-1.5"
								onclick={() => {
									mode = 'set_pin';
									errorMsg = '';
								}}
							>
								<Shield class="size-3.5" />
								Buat PIN Baru
							</Button>
						{/if}
					</div>
				</div>

				<!-- Auto Lock Options Section -->
				<div class="space-y-2.5">
					<div class="flex items-center justify-between">
						<label class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
							<Clock class="size-3.5 text-primary" />
							Kunci Otomatis saat Minimize
						</label>
						{#if !pinSet}
							<span class="text-[10px] text-amber-500 font-medium">Perlu buat PIN</span>
						{/if}
					</div>

					<div class="grid grid-cols-1 gap-1.5">
						{#each AUTOLOCK_OPTIONS as opt (opt.value)}
							{@const isSelected = autoLockDelay === opt.value}
							<button
								type="button"
								class={cn(
									'flex items-center justify-between p-2.5 rounded-xl border text-left transition-all cursor-pointer',
									isSelected
										? 'border-primary bg-primary/10 text-foreground ring-1 ring-primary/30 shadow-xs'
										: 'border-border/60 bg-card hover:bg-muted/50 text-foreground'
								)}
								onclick={() => handleSelectDelay(opt.value)}
							>
								<div class="min-w-0 flex-1 pr-2">
									<div class="flex items-center gap-2">
										<span class="text-xs font-semibold">{opt.label}</span>
										{#if opt.value === '60'}
											<span class="text-[9px] font-medium px-1.5 py-0.2 rounded-full bg-primary/20 text-primary">
												Disarankan
											</span>
										{/if}
									</div>
									<p class="text-[11px] text-muted-foreground mt-0.5">{opt.desc}</p>
								</div>
								<div class={cn(
									'size-4 rounded-full border flex items-center justify-center shrink-0 transition-colors',
									isSelected ? 'border-primary bg-primary text-primary-foreground' : 'border-muted-foreground/40'
								)}>
									{#if isSelected}
										<div class="size-1.5 rounded-full bg-white"></div>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				</div>

				<!-- Info Box: Preserves state -->
				<div class="p-2.5 rounded-xl border border-primary/20 bg-primary/5 text-xs text-muted-foreground flex items-start gap-2.5">
					<Sparkles class="size-4 text-primary shrink-0 mt-0.5" />
					<div class="text-[11px] leading-relaxed">
						<span class="font-bold text-foreground">Tetap di Halaman Terakhir:</span>
						Ketika aplikasi terkunci otomatis, space, kategori, dan dialog todo yang sedang dibuka akan tetap tersimpan. Setelah PIN dimasukkan, Anda langsung kembali ke tampilan semula.
					</div>
				</div>

			{:else if mode === 'set_pin'}
				<!-- Set PIN Mode -->
				<div class="space-y-3">
					<div>
						<h3 class="text-xs font-bold text-foreground">Buat PIN Baru</h3>
						<p class="text-[11px] text-muted-foreground">Minimal 4 digit angka atau karakter.</p>
					</div>

					<div class="space-y-2">
						<Input
							type="password"
							placeholder="Masukkan PIN Baru"
							bind:value={newPin}
							class="text-center text-sm"
							autofocus
						/>
						<Input
							type="password"
							placeholder="Ketik Ulang PIN Baru"
							bind:value={confirmPin}
							class="text-center text-sm"
							onkeydown={(e) => e.key === 'Enter' && handleSetPin()}
						/>
					</div>

					{#if errorMsg}
						<p class="text-center text-xs text-destructive">{errorMsg}</p>
					{/if}

					<div class="flex items-center gap-2 pt-2">
						<Button
							variant="outline"
							size="sm"
							class="flex-1 text-xs"
							onclick={() => {
								mode = 'main';
								errorMsg = '';
							}}
							disabled={isBusy}
						>
							Batal
						</Button>
						<Button
							size="sm"
							class="flex-1 text-xs"
							onclick={handleSetPin}
							disabled={isBusy}
						>
							{isBusy ? 'Menyimpan...' : 'Simpan PIN'}
						</Button>
					</div>
				</div>

			{:else if mode === 'change_pin'}
				<!-- Change PIN Mode -->
				<div class="space-y-3">
					<div>
						<h3 class="text-xs font-bold text-foreground">Ganti PIN</h3>
						<p class="text-[11px] text-muted-foreground">Masukkan PIN lama untuk verifikasi.</p>
					</div>

					<div class="space-y-2">
						<Input
							type="password"
							placeholder="PIN Lama"
							bind:value={oldPin}
							class="text-center text-sm"
							autofocus
						/>
						<Input
							type="password"
							placeholder="PIN Baru (min 4 digit)"
							bind:value={newPin}
							class="text-center text-sm"
						/>
						<Input
							type="password"
							placeholder="Konfirmasi PIN Baru"
							bind:value={confirmPin}
							class="text-center text-sm"
							onkeydown={(e) => e.key === 'Enter' && handleChangePin()}
						/>
					</div>

					{#if errorMsg}
						<p class="text-center text-xs text-destructive">{errorMsg}</p>
					{/if}

					<div class="flex items-center gap-2 pt-2">
						<Button
							variant="outline"
							size="sm"
							class="flex-1 text-xs"
							onclick={() => {
								mode = 'main';
								errorMsg = '';
							}}
							disabled={isBusy}
						>
							Batal
						</Button>
						<Button
							size="sm"
							class="flex-1 text-xs"
							onclick={handleChangePin}
							disabled={isBusy}
						>
							{isBusy ? 'Memproses...' : 'Ubah PIN'}
						</Button>
					</div>
				</div>

			{:else if mode === 'clear_pin'}
				<!-- Clear PIN Mode -->
				<div class="space-y-3">
					<div>
						<h3 class="text-xs font-bold text-destructive">Hapus Proteksi PIN</h3>
						<p class="text-[11px] text-muted-foreground">Masukkan PIN untuk menghapus penguncian aplikasi.</p>
					</div>

					<Input
						type="password"
						placeholder="Masukkan PIN saat ini"
						bind:value={oldPin}
						class="text-center text-sm"
						autofocus
						onkeydown={(e) => e.key === 'Enter' && handleClearPin()}
					/>

					{#if errorMsg}
						<p class="text-center text-xs text-destructive">{errorMsg}</p>
					{/if}

					<div class="flex items-center gap-2 pt-2">
						<Button
							variant="outline"
							size="sm"
							class="flex-1 text-xs"
							onclick={() => {
								mode = 'main';
								errorMsg = '';
							}}
							disabled={isBusy}
						>
							Batal
						</Button>
						<Button
							variant="destructive"
							size="sm"
							class="flex-1 text-xs"
							onclick={handleClearPin}
							disabled={isBusy}
						>
							{isBusy ? 'Memproses...' : 'Hapus PIN'}
						</Button>
					</div>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		{#if mode === 'main'}
			<div class="flex items-center justify-between p-3 border-t border-border/70 bg-muted/20">
				<Button
					variant="outline"
					size="sm"
					class="text-xs gap-1.5 cursor-pointer text-muted-foreground hover:text-foreground"
					onclick={() => {
						open = false;
						if (onLockNow) onLockNow();
					}}
					disabled={!pinSet}
				>
					<Lock class="size-3.5" />
					Kunci Sekarang
				</Button>

				<Button
					size="sm"
					class="text-xs px-4 cursor-pointer"
					onclick={() => (open = false)}
				>
					Selesai
				</Button>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
