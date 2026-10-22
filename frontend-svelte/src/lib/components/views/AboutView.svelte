<script lang="ts">
	import {
		ArrowLeft,
		ExternalLink,
		Copy,
		Check,
		ShieldCheck,
		Shield,
		ShieldAlert,
		Sparkles,
		Heart,
		Clock,
		Settings2,
		Lock
	} from '@lucide/svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import * as Select from '#lib/components/ui/select/index.js';
	import { AUTOLOCK_OPTIONS, AUTOLOCK_KEY } from '#lib/types.js';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';

	let {
		onBack,
		autoLockDelay = $bindable('never'),
		pinSet = false,
		onOpenSecuritySettings,
		onLockApp
	}: {
		onBack: () => void;
		autoLockDelay?: string;
		pinSet?: boolean;
		onOpenSecuritySettings?: () => void;
		onLockApp?: () => void;
	} = $props();

	const GITHUB_URL = 'https://github.com/rohmatext';
	let copied = $state(false);

	function handleAutoLockChange(val: string) {
		autoLockDelay = val;
		localStorage.setItem(AUTOLOCK_KEY, val);
		const opt = AUTOLOCK_OPTIONS.find((o) => o.value === val);
		toast.success(`Kunci otomatis: ${opt?.label ?? val}`);
	}

	async function openGithub() {
		try {
			await invoke('open_url', { url: GITHUB_URL });
		} catch {
			window.open(GITHUB_URL, '_blank');
		}
	}

	async function copyGithub() {
		try {
			await navigator.clipboard.writeText(GITHUB_URL);
			copied = true;
			toast.success('Link GitHub disalin ke clipboard');
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			toast.error('Gagal menyalin link');
		}
	}
</script>

<div class="flex h-full flex-col p-3 overflow-hidden">
	<!-- Header -->
	<header class="flex items-center gap-2 pb-3 border-b border-border/70 shrink-0">
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button {...props} variant="outline" size="icon-sm" onclick={onBack}>
						<ArrowLeft class="size-4" />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Kembali</Tooltip.Content>
		</Tooltip.Root>
		<div class="min-w-0 flex-1">
			<h1 class="text-sm font-bold tracking-tight">Tentang Aplikasi</h1>
			<p class="text-[11px] text-muted-foreground truncate">Informasi versi dan pengembang</p>
		</div>
	</header>

	<!-- Content -->
	<main class="flex-1 overflow-y-auto pr-1 pt-3 space-y-3.5">
		<!-- App Hero Card -->
		<div class="flex flex-col items-center text-center p-4 rounded-xl border border-border/60 bg-gradient-to-b from-muted/50 to-muted/20">
			<div class="size-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center text-primary mb-2 shadow-xs">
				<Sparkles class="size-6" />
			</div>
			<div class="flex items-center gap-2">
				<h2 class="text-sm font-bold">CheckMat</h2>
				<Badge variant="outline" class="text-[10px] px-1.5 py-0 h-4 border-primary/30 text-primary bg-primary/5">
					v0.1.0
				</Badge>
			</div>
			<p class="text-[11px] text-muted-foreground mt-1 max-w-xs">
				Minimalist, fast & secure desktop todo manager untuk produktivitas harian.
			</p>
		</div>

		<!-- Credit & Author Card -->
		<div class="rounded-xl border border-border/70 bg-card p-3.5 space-y-3">
			<div class="flex items-center justify-between">
				<span class="text-[11px] font-semibold text-muted-foreground tracking-wider uppercase">
					Pengembang & Pembuat
				</span>
				<Badge variant="secondary" class="text-[10px] gap-1 px-1.5">
					<Heart class="size-2.5 text-rose-500 fill-rose-500" /> Open Source
				</Badge>
			</div>

			<div class="flex items-center gap-3">
				<div class="size-10 rounded-full bg-primary/10 border border-primary/25 flex items-center justify-center text-primary shrink-0 font-bold text-sm">
					MR
				</div>
				<div class="min-w-0 flex-1">
					<p class="text-xs font-bold text-foreground">Muhammad Rohmat Abdullah</p>
					<p class="text-[11px] text-muted-foreground truncate">@rohmatext</p>
				</div>
			</div>

			<div class="flex items-center gap-2 pt-1">
				<Button
					size="sm"
					class="flex-1 h-8 text-xs gap-1.5 cursor-pointer shadow-xs"
					onclick={openGithub}
				>
					<svg viewBox="0 0 24 24" class="size-3.5 fill-current shrink-0" aria-hidden="true">
						<path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
					</svg>
					Buka GitHub
					<ExternalLink class="size-3 opacity-70" />
				</Button>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="outline"
								size="icon-sm"
								class="h-8 w-8 shrink-0 cursor-pointer"
								onclick={copyGithub}
							>
								{#if copied}
									<Check class="size-3.5 text-emerald-500" />
								{:else}
									<Copy class="size-3.5" />
								{/if}
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>{copied ? 'Tersalin!' : 'Salin tautan profil'}</Tooltip.Content>
				</Tooltip.Root>
			</div>
		</div>

		<!-- Security & Auto-Lock Card -->
		<div class="rounded-xl border border-border/70 bg-card p-3.5 space-y-3">
			<div class="flex items-center justify-between">
				<span class="text-[11px] font-semibold text-muted-foreground tracking-wider uppercase flex items-center gap-1.5">
					<Shield class="size-3.5 text-primary" />
					Keamanan & Kunci Otomatis
				</span>
				{#if pinSet}
					<Badge variant="outline" class="text-[10px] gap-1 px-1.5 py-0 border-emerald-500/30 text-emerald-600 dark:text-emerald-400 bg-emerald-500/10">
						<Check class="size-2.5" /> PIN Aktif
					</Badge>
				{:else}
					<Badge variant="outline" class="text-[10px] gap-1 px-1.5 py-0 border-amber-500/30 text-amber-600 dark:text-amber-400 bg-amber-500/10">
						Belum Ada PIN
					</Badge>
				{/if}
			</div>

			<div class="space-y-1.5">
				<span class="text-xs font-medium text-foreground flex items-center gap-1.5">
					<Clock class="size-3.5 text-muted-foreground" />
					Kunci saat Minimize:
				</span>
				<Select.Root
					type="single"
					bind:value={autoLockDelay}
					items={AUTOLOCK_OPTIONS.map((o) => ({ value: o.value, label: `${o.label} — ${o.desc}` }))}
					onValueChange={(val) => {
						if (val) handleAutoLockChange(val);
					}}
				>
					<Select.Trigger class="w-full h-8 text-xs rounded-lg justify-between cursor-pointer border-border/80 bg-background shadow-xs">
						<Select.Value placeholder="Pilih durasi kunci otomatis" />
					</Select.Trigger>
					<Select.Content class="rounded-xl border border-border/80 p-1 shadow-xl max-h-60">
						<Select.Group>
							{#each AUTOLOCK_OPTIONS as opt (opt.value)}
								<Select.Item
									value={opt.value}
									label={`${opt.label} — ${opt.desc}`}
									class="rounded-lg text-xs py-1.5 px-2.5 cursor-pointer"
								>
									<div class="flex items-center gap-2">
										<span class="font-medium text-foreground">{opt.label}</span>
										<span class="text-[11px] text-muted-foreground truncate">— {opt.desc}</span>
									</div>
								</Select.Item>
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
			</div>

			<div class="flex items-center gap-2 pt-1">
				{#if onOpenSecuritySettings}
					<Button
						variant="outline"
						size="sm"
						class="flex-1 h-8 text-xs gap-1.5 cursor-pointer shadow-xs"
						onclick={onOpenSecuritySettings}
					>
						<Settings2 class="size-3.5" />
						Kelola PIN & Pengaturan
					</Button>
				{/if}
				{#if onLockApp && pinSet}
					<Button
						variant="secondary"
						size="sm"
						class="h-8 text-xs gap-1.5 cursor-pointer shadow-xs"
						onclick={onLockApp}
					>
						<Lock class="size-3.5" />
						Kunci Sekarang
					</Button>
				{/if}
			</div>
		</div>

		<!-- Privacy & Security -->
		<div class="flex items-center gap-2 p-2.5 rounded-lg border border-border/60 bg-muted/25 text-xs text-muted-foreground">
			<ShieldCheck class="size-4 text-emerald-500 shrink-0" />
			<span class="text-[11px] leading-relaxed">
				Data tersimpan 100% lokal di komputer Anda. Tanpa analitik, tanpa pelacakan pihak ketiga.
			</span>
		</div>
	</main>
</div>
