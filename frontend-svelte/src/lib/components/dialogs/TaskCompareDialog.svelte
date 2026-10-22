<script lang="ts">
	import {
		ArrowLeftRight,
		Check,
		Circle,
		Clock,
		Columns2,
		Copy,
		ListChecks,
		Rows2,
		Search,
		Trash,
		Undo2,
		X
	} from '@lucide/svelte';
	import { Button, buttonVariants } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import * as Dialog from '#lib/components/ui/dialog/index.js';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from 'svelte-sonner';
	import {
		formatFullDate,
		getBadgeCls,
		type Category,
		type Task,
		type TaskEvent,
		type Subtask
	} from '#lib/types.js';
	import CopyableText from '#lib/components/CopyableText.svelte';
	import { cn } from '#lib/utils';

	let {
		open = $bindable(false),
		taskA = $bindable<Task | null>(null),
		taskB = $bindable<Task | null>(null),
		tasks = [],
		cats = [],
		onUpdateTask,
		onRefreshTasks
	}: {
		open: boolean;
		taskA: Task | null;
		taskB: Task | null;
		tasks: Task[];
		cats: Category[];
		onUpdateTask?: (id: number, text: string, catId: number, description?: string) => Promise<void>;
		onRefreshTasks?: () => Promise<void> | void;
	} = $props();

	let layout = $state<'split' | 'stacked'>('split');
	let pickingTarget = $state<'A' | 'B' | null>(null);
	let pickerQuery = $state('');

	let eventsA = $state<TaskEvent[]>([]);
	let eventsB = $state<TaskEvent[]>([]);
	let subtasksA = $state<Subtask[]>([]);
	let subtasksB = $state<Subtask[]>([]);

	async function loadSubtasksA() {
		if (taskA) {
			try {
				subtasksA = (await invoke<Subtask[]>('list_subtasks', { taskId: taskA.id })) || [];
			} catch {
				subtasksA = [];
			}
		} else {
			subtasksA = [];
		}
	}

	async function loadSubtasksB() {
		if (taskB) {
			try {
				subtasksB = (await invoke<Subtask[]>('list_subtasks', { taskId: taskB.id })) || [];
			} catch {
				subtasksB = [];
			}
		} else {
			subtasksB = [];
		}
	}

	// If Task B is empty when opened, prompt to pick Task B
	$effect(() => {
		if (open) {
			pickerQuery = '';
			if (taskA && !taskB) {
				pickingTarget = 'B';
			} else if (!taskA && !taskB) {
				pickingTarget = 'A';
			} else {
				pickingTarget = null;
			}
		} else {
			pickingTarget = null;
		}
	});

	// Load events & subtasks for Task A
	$effect(() => {
		if (open && taskA) {
			invoke<TaskEvent[]>('list_task_events', { taskId: taskA.id })
				.then((res) => (eventsA = res || []))
				.catch(() => (eventsA = []));
			loadSubtasksA();
		} else {
			eventsA = [];
			subtasksA = [];
		}
	});

	// Load events & subtasks for Task B
	$effect(() => {
		if (open && taskB) {
			invoke<TaskEvent[]>('list_task_events', { taskId: taskB.id })
				.then((res) => (eventsB = res || []))
				.catch(() => (eventsB = []));
			loadSubtasksB();
		} else {
			eventsB = [];
			subtasksB = [];
		}
	});

	async function copySubtask(sourceSubtask: Subtask, targetTask: Task | null) {
		if (!targetTask) return;
		try {
			await invoke('add_subtask', {
				taskId: targetTask.id,
				title: sourceSubtask.title
			});
			if (targetTask.id === taskA?.id) {
				await loadSubtasksA();
			} else if (targetTask.id === taskB?.id) {
				await loadSubtasksB();
			}
			await onRefreshTasks?.();
			toast.success(`Subtask "${sourceSubtask.title}" berhasil disalin ke Todo ${targetTask.id === taskA?.id ? 'A' : 'B'}`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function copyAllSubtasks(sourceSubtasks: Subtask[], targetTask: Task | null) {
		if (!targetTask || sourceSubtasks.length === 0) return;
		try {
			for (const st of sourceSubtasks) {
				await invoke('add_subtask', {
					taskId: targetTask.id,
					title: st.title
				});
			}
			if (targetTask.id === taskA?.id) {
				await loadSubtasksA();
			} else if (targetTask.id === taskB?.id) {
				await loadSubtasksB();
			}
			await onRefreshTasks?.();
			toast.success(`${sourceSubtasks.length} subtask berhasil disalin ke Todo ${targetTask.id === taskA?.id ? 'A' : 'B'}`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function copyTitle(from: 'A' | 'B') {
		const source = from === 'A' ? taskA : taskB;
		const target = from === 'A' ? taskB : taskA;
		if (!source || !target || !onUpdateTask) return;
		try {
			await onUpdateTask(target.id, source.text, target.category_id, target.description);
			target.text = source.text;
			await onRefreshTasks?.();
			toast.success(`Judul disalin ke Todo ${from === 'A' ? 'B' : 'A'}`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function copyDescription(from: 'A' | 'B') {
		const source = from === 'A' ? taskA : taskB;
		const target = from === 'A' ? taskB : taskA;
		if (!source || !target || !onUpdateTask) return;
		const newDesc = source.description ?? '';
		try {
			await onUpdateTask(target.id, target.text, target.category_id, newDesc);
			target.description = newDesc;
			await onRefreshTasks?.();
			toast.success(`Deskripsi disalin ke Todo ${from === 'A' ? 'B' : 'A'}`);
		} catch (e) {
			toast.error(String(e));
		}
	}

	const catA = $derived(cats.find((c) => c.id === taskA?.category_id));
	const catB = $derived(cats.find((c) => c.id === taskB?.category_id));

	const filteredPickerTasks = $derived.by(() => {
		const otherId = pickingTarget === 'A' ? taskB?.id : taskA?.id;
		const q = pickerQuery.trim().toLowerCase();
		return tasks.filter((t) => {
			if (otherId && t.id === otherId) return false;
			if (!q) return true;
			const cat = cats.find((c) => c.id === t.category_id);
			return (
				t.text.toLowerCase().includes(q) ||
				(t.description ?? '').toLowerCase().includes(q) ||
				(cat?.name ?? '').toLowerCase().includes(q)
			);
		});
	});

	function selectTaskForTarget(t: Task) {
		if (pickingTarget === 'A') {
			taskA = t;
			if (!taskB) {
				pickingTarget = 'B';
				pickerQuery = '';
				return;
			}
		} else if (pickingTarget === 'B') {
			taskB = t;
		}
		pickingTarget = null;
		pickerQuery = '';
	}

	function swapTasks() {
		const temp = taskA;
		taskA = taskB;
		taskB = temp;
	}

	function getStatusLabel(t: Task | null): string {
		if (!t) return '-';
		if (t.deleted) return 'Dihapus';
		if (t.done || t.status === 'done') return 'Selesai';
		if (t.status === 'inprogress') return 'In Progress';
		return 'Todo';
	}

	function getStatusColorClass(t: Task | null): string {
		if (!t) return '';
		if (t.deleted) return 'bg-destructive/10 text-destructive border-destructive/30';
		if (t.done || t.status === 'done') return 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30';
		if (t.status === 'inprogress') return 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30';
		return 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/30';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-w-md w-[96vw] max-h-[90vh] flex flex-col p-4 gap-3 overflow-hidden">
		<!-- Header -->
		<Dialog.Header class="gap-1 shrink-0 pb-1 border-b border-border/60">
			<div class="flex items-center justify-between gap-2 flex-wrap">
				<Dialog.Title class="text-sm font-semibold flex items-center gap-1.5">
					<ArrowLeftRight class="size-4 text-primary" />
					<span>Bandingkan Todo</span>
				</Dialog.Title>

				{#if !pickingTarget && taskA && taskB}
					<div class="flex items-center gap-1 mr-5">
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon-xs"
										class="size-6 text-muted-foreground hover:text-foreground"
										onclick={swapTasks}
									>
										<ArrowLeftRight class="size-3" />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Tukar posisi Todo A dan B</Tooltip.Content>
						</Tooltip.Root>

						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon-xs"
										class="size-6 text-muted-foreground hover:text-foreground"
										onclick={() => (layout = layout === 'split' ? 'stacked' : 'split')}
									>
										{#if layout === 'split'}
											<Rows2 class="size-3" />
										{:else}
											<Columns2 class="size-3" />
										{/if}
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Tampilan: {layout === 'split' ? 'Ganti ke Bertumpuk' : 'Ganti ke Berdampingan'}</Tooltip.Content>
						</Tooltip.Root>
					</div>
				{/if}
			</div>
			<Dialog.Description class="text-xs text-muted-foreground">
				{pickingTarget
					? `Pilih todo untuk ${pickingTarget === 'A' ? 'Todo A (Kiri)' : 'Todo B (Kanan)'}`
					: 'Perbandingan detail atribut, kategori, dan deskripsi antar todo.'}
			</Dialog.Description>
		</Dialog.Header>

		<!-- Mode Pemilihan Todo -->
		{#if pickingTarget}
			<div class="flex flex-col gap-2 flex-1 min-h-0">
				<div class="relative flex items-center shrink-0">
					<Search class="absolute left-2.5 size-3.5 text-muted-foreground pointer-events-none" />
					<Input
						bind:value={pickerQuery}
						placeholder="Cari nama todo atau kategori..."
						class="h-8 pl-8 pr-7 text-xs"
					/>
					{#if pickerQuery}
						<button
							type="button"
							onclick={() => (pickerQuery = '')}
							class="absolute right-2 text-muted-foreground hover:text-foreground cursor-pointer"
						>
							<X class="size-3" />
						</button>
					{/if}
				</div>

				<div class="flex-1 overflow-y-auto space-y-1 pr-0.5 min-h-52 max-h-[50vh]">
					{#each filteredPickerTasks as t (t.id)}
						{@const c = cats.find((cat) => cat.id === t.category_id)}
						<button
							type="button"
							class="flex w-full items-center justify-between gap-2 p-2 rounded-lg border border-border/60 hover:border-primary/50 hover:bg-muted/50 text-left transition-colors cursor-pointer group"
							onclick={() => selectTaskForTarget(t)}
						>
							<div class="min-w-0 flex-1">
								<p class="text-xs font-medium text-foreground break-words [overflow-wrap:anywhere] group-hover:text-primary leading-snug">
									{t.text}
								</p>
								<div class="flex items-center gap-1.5 mt-0.5 text-[10px] text-muted-foreground flex-wrap">
									{#if c}
										<Badge variant="outline" class={cn('text-[9px] px-1 py-0 h-3.5', getBadgeCls(c.color))}>
											{c.name}
										</Badge>
									{/if}
									<span class={cn('text-[9px] px-1 py-0 rounded border', getStatusColorClass(t))}>
										{getStatusLabel(t)}
									</span>
								</div>
							</div>
						</button>
					{/each}
					{#if filteredPickerTasks.length === 0}
						<p class="py-8 text-center text-xs text-muted-foreground">
							{tasks.length === 0 ? 'Belum ada todo.' : 'Todo tidak ditemukan.'}
						</p>
					{/if}
				</div>

				{#if taskA || taskB}
					<div class="pt-2 border-t border-border flex justify-end shrink-0">
						<Button
							variant="ghost"
							size="xs"
							class="text-xs"
							onclick={() => (pickingTarget = null)}
						>
							Kembali ke Perbandingan
						</Button>
					</div>
				{/if}
			</div>

		<!-- Mode Tampilan Perbandingan -->
		{:else if taskA && taskB}
			<!-- Task Selectors & Swap Bar -->
			<div class="grid grid-cols-2 gap-2 shrink-0">
				<!-- Selector A -->
				<div class="rounded-lg border border-primary/30 bg-primary/5 p-2 flex flex-col gap-1 min-w-0">
					<div class="flex items-center justify-between">
						<span class="text-[10px] font-bold text-primary tracking-wider uppercase">Todo A</span>
						<button
							type="button"
							class="text-[10px] text-muted-foreground hover:text-foreground underline cursor-pointer"
							onclick={() => {
								pickingTarget = 'A';
								pickerQuery = '';
							}}
						>
							Ganti
						</button>
					</div>
					<p class="text-xs font-semibold break-words [overflow-wrap:anywhere] line-clamp-2 leading-tight text-foreground select-text" title={taskA.text}>
						{taskA.text}
					</p>
				</div>

				<!-- Selector B -->
				<div class="rounded-lg border border-purple-500/30 bg-purple-500/5 p-2 flex flex-col gap-1 min-w-0">
					<div class="flex items-center justify-between">
						<span class="text-[10px] font-bold text-purple-600 dark:text-purple-400 tracking-wider uppercase">Todo B</span>
						<button
							type="button"
							class="text-[10px] text-muted-foreground hover:text-foreground underline cursor-pointer"
							onclick={() => {
								pickingTarget = 'B';
								pickerQuery = '';
							}}
						>
							Ganti
						</button>
					</div>
					<p class="text-xs font-semibold break-words [overflow-wrap:anywhere] line-clamp-2 leading-tight text-foreground select-text" title={taskB.text}>
						{taskB.text}
					</p>
				</div>
			</div>

			<!-- Quick Difference Summary Pills -->
			<div class="flex items-center justify-between gap-1.5 px-1.5 text-[10px] text-muted-foreground border-y border-border/50 py-1 shrink-0 bg-muted/20 rounded flex-wrap">
				<span class="shrink-0 font-medium">Perbedaan:</span>
				<div class="flex items-center gap-1.5 flex-wrap">
					<span class={cn('px-1.5 py-0.2 rounded border font-medium', taskA.category_id === taskB.category_id ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20')}>
						Kategori: {taskA.category_id === taskB.category_id ? 'Sama' : 'Beda'}
					</span>
					<span class={cn('px-1.5 py-0.2 rounded border font-medium', getStatusLabel(taskA) === getStatusLabel(taskB) ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20')}>
						Status: {getStatusLabel(taskA) === getStatusLabel(taskB) ? 'Sama' : 'Beda'}
					</span>
					<span class={cn('px-1.5 py-0.2 rounded border font-medium', (taskA.description ?? '') === (taskB.description ?? '') ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20')}>
						Deskripsi: {(taskA.description ?? '') === (taskB.description ?? '') ? 'Sama' : 'Beda'}
					</span>
					<span class={cn('px-1.5 py-0.2 rounded border font-medium', subtasksA.length === subtasksB.length ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20')}>
						Subtask: {subtasksA.length} vs {subtasksB.length}
					</span>
				</div>
			</div>

			<!-- Comparison Content Body -->
			<div class="flex-1 overflow-y-auto space-y-3 pr-0.5 text-xs">
				<!-- Teks Judul -->
				<div class="space-y-1">
					<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">Teks / Judul</span>
					<div class={cn('gap-2', layout === 'split' ? 'grid grid-cols-2' : 'space-y-1.5')}>
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 min-w-0">
							<div class="flex items-center justify-between mb-0.5">
								<span class="text-[9px] text-muted-foreground font-bold">Todo A</span>
								{#if onUpdateTask}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="icon-xs"
													class="size-5 text-muted-foreground hover:text-primary cursor-pointer"
													onclick={() => copyTitle('A')}
												>
													<Copy class="size-3" />
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin judul ini ke Todo B</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>
							<p class="text-xs break-words [overflow-wrap:anywhere] font-medium leading-relaxed select-text">{taskA.text}</p>
						</div>
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 min-w-0">
							<div class="flex items-center justify-between mb-0.5">
								<span class="text-[9px] text-muted-foreground font-bold">Todo B</span>
								{#if onUpdateTask}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="icon-xs"
													class="size-5 text-muted-foreground hover:text-purple-600 dark:hover:text-purple-400 cursor-pointer"
													onclick={() => copyTitle('B')}
												>
													<Copy class="size-3" />
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin judul ini ke Todo A</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>
							<p class="text-xs break-words [overflow-wrap:anywhere] font-medium leading-relaxed select-text">{taskB.text}</p>
						</div>
					</div>
				</div>

				<!-- Kategori & Status -->
				<div class="space-y-1">
					<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">Kategori & Status</span>
					<div class={cn('gap-2', layout === 'split' ? 'grid grid-cols-2' : 'space-y-1.5')}>
						<!-- A -->
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 flex flex-col gap-1.5 min-w-0">
							<span class="text-[9px] text-muted-foreground font-bold">Todo A</span>
							<div class="flex items-center gap-1.5 flex-wrap min-w-0">
								{#if catA}
									<Badge variant="outline" class={cn('text-[10px] font-medium max-w-full break-words [overflow-wrap:anywhere]', getBadgeCls(catA.color))}>
										{catA.name}
									</Badge>
								{:else}
									<span class="text-muted-foreground text-[10px]">Tanpa kategori</span>
								{/if}
								<span class={cn('text-[10px] px-1.5 py-0.5 rounded border font-medium', getStatusColorClass(taskA))}>
									{getStatusLabel(taskA)}
								</span>
							</div>
						</div>
						<!-- B -->
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 flex flex-col gap-1.5 min-w-0">
							<span class="text-[9px] text-muted-foreground font-bold">Todo B</span>
							<div class="flex items-center gap-1.5 flex-wrap min-w-0">
								{#if catB}
									<Badge variant="outline" class={cn('text-[10px] font-medium max-w-full break-words [overflow-wrap:anywhere]', getBadgeCls(catB.color))}>
										{catB.name}
									</Badge>
								{:else}
									<span class="text-muted-foreground text-[10px]">Tanpa kategori</span>
								{/if}
								<span class={cn('text-[10px] px-1.5 py-0.5 rounded border font-medium', getStatusColorClass(taskB))}>
									{getStatusLabel(taskB)}
								</span>
							</div>
						</div>
					</div>
				</div>

				<!-- Deskripsi -->
				<div class="space-y-1">
					<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">Deskripsi</span>
					<div class={cn('gap-2', layout === 'split' ? 'grid grid-cols-2' : 'space-y-1.5')}>
						<!-- A -->
						<div class="p-2 rounded-md bg-muted/30 border border-border/60 min-w-0 max-h-40 overflow-y-auto">
							<div class="flex items-center justify-between mb-0.5">
								<span class="text-[9px] text-muted-foreground font-bold">Todo A</span>
								{#if onUpdateTask && taskA.description?.trim()}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="icon-xs"
													class="size-5 text-muted-foreground hover:text-primary cursor-pointer"
													onclick={() => copyDescription('A')}
												>
													<Copy class="size-3" />
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin deskripsi ini ke Todo B</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>
							{#if taskA.description?.trim()}
								<div class="text-xs text-foreground/90 leading-relaxed break-words whitespace-pre-wrap">
									<CopyableText text={taskA.description} />
								</div>
							{:else}
								<span class="text-muted-foreground italic text-[11px]">(Tidak ada deskripsi)</span>
							{/if}
						</div>
						<!-- B -->
						<div class="p-2 rounded-md bg-muted/30 border border-border/60 min-w-0 max-h-40 overflow-y-auto">
							<div class="flex items-center justify-between mb-0.5">
								<span class="text-[9px] text-muted-foreground font-bold">Todo B</span>
								{#if onUpdateTask && taskB.description?.trim()}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="icon-xs"
													class="size-5 text-muted-foreground hover:text-purple-600 dark:hover:text-purple-400 cursor-pointer"
													onclick={() => copyDescription('B')}
												>
													<Copy class="size-3" />
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin deskripsi ini ke Todo A</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>
							{#if taskB.description?.trim()}
								<div class="text-xs text-foreground/90 leading-relaxed break-words whitespace-pre-wrap">
									<CopyableText text={taskB.description} />
								</div>
							{:else}
								<span class="text-muted-foreground italic text-[11px]">(Tidak ada deskripsi)</span>
							{/if}
						</div>
					</div>
				</div>

				<!-- Subtask -->
				<div class="space-y-1">
					<div class="flex items-center justify-between">
						<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">
							Subtask ({subtasksA.length} vs {subtasksB.length})
						</span>
					</div>

					<div class={cn('gap-2', layout === 'split' ? 'grid grid-cols-2' : 'space-y-1.5')}>
						<!-- Subtasks A -->
						<div class="p-2 rounded-md bg-muted/30 border border-border/60 min-w-0 flex flex-col gap-1.5">
							<div class="flex items-center justify-between gap-1 flex-wrap">
								<span class="text-[9px] text-muted-foreground font-bold">
									Todo A ({subtasksA.filter(s => s.done).length}/{subtasksA.length})
								</span>
								{#if subtasksA.length > 0}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="xs"
													class="h-5 px-1.5 text-[9px] text-primary hover:text-primary gap-1"
													onclick={() => copyAllSubtasks(subtasksA, taskB)}
												>
													<Copy class="size-2.5" />
													<span>Salin Semua ke B</span>
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin semua subtask Todo A ke Todo B</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>

							{#if subtasksA.length > 0}
								<div class="space-y-1 max-h-36 overflow-y-auto pr-0.5">
									{#each subtasksA as st (st.id)}
										<div class="flex items-center justify-between gap-1.5 p-1 rounded bg-background/70 border border-border/50 text-[11px] group">
											<div class="flex items-center gap-1.5 min-w-0 flex-1">
												<span class={cn('size-1.5 rounded-full shrink-0', st.done ? 'bg-emerald-500' : 'bg-muted-foreground/40')}></span>
												<span class={cn('break-words [overflow-wrap:anywhere] flex-1 select-text leading-tight text-xs', st.done && 'line-through text-muted-foreground')}>
													{st.title}
												</span>
											</div>
											<Tooltip.Root>
												<Tooltip.Trigger>
													{#snippet child({ props })}
														<Button
															{...props}
															variant="ghost"
															size="icon-xs"
															class="size-5 shrink-0 text-muted-foreground hover:text-primary"
															onclick={() => copySubtask(st, taskB)}
														>
															<Copy class="size-3" />
														</Button>
													{/snippet}
												</Tooltip.Trigger>
												<Tooltip.Content>Salin subtask ini ke Todo B</Tooltip.Content>
											</Tooltip.Root>
										</div>
									{/each}
								</div>
							{:else}
								<span class="text-muted-foreground italic text-[11px]">(Tidak ada subtask)</span>
							{/if}
						</div>

						<!-- Subtasks B -->
						<div class="p-2 rounded-md bg-muted/30 border border-border/60 min-w-0 flex flex-col gap-1.5">
							<div class="flex items-center justify-between gap-1 flex-wrap">
								<span class="text-[9px] text-muted-foreground font-bold">
									Todo B ({subtasksB.filter(s => s.done).length}/{subtasksB.length})
								</span>
								{#if subtasksB.length > 0}
									<Tooltip.Root>
										<Tooltip.Trigger>
											{#snippet child({ props })}
												<Button
													{...props}
													variant="ghost"
													size="xs"
													class="h-5 px-1.5 text-[9px] text-purple-600 dark:text-purple-400 hover:text-purple-700 gap-1"
													onclick={() => copyAllSubtasks(subtasksB, taskA)}
												>
													<Copy class="size-2.5" />
													<span>Salin Semua ke A</span>
												</Button>
											{/snippet}
										</Tooltip.Trigger>
										<Tooltip.Content>Salin semua subtask Todo B ke Todo A</Tooltip.Content>
									</Tooltip.Root>
								{/if}
							</div>

							{#if subtasksB.length > 0}
								<div class="space-y-1 max-h-36 overflow-y-auto pr-0.5">
									{#each subtasksB as st (st.id)}
										<div class="flex items-center justify-between gap-1.5 p-1 rounded bg-background/70 border border-border/50 text-[11px] group">
											<div class="flex items-center gap-1.5 min-w-0 flex-1">
												<span class={cn('size-1.5 rounded-full shrink-0', st.done ? 'bg-emerald-500' : 'bg-muted-foreground/40')}></span>
												<span class={cn('break-words [overflow-wrap:anywhere] flex-1 select-text leading-tight text-xs', st.done && 'line-through text-muted-foreground')}>
													{st.title}
												</span>
											</div>
											<Tooltip.Root>
												<Tooltip.Trigger>
													{#snippet child({ props })}
														<Button
															{...props}
															variant="ghost"
															size="icon-xs"
															class="size-5 shrink-0 text-muted-foreground hover:text-purple-600 dark:hover:text-purple-400"
															onclick={() => copySubtask(st, taskA)}
														>
															<Copy class="size-3" />
														</Button>
													{/snippet}
												</Tooltip.Trigger>
												<Tooltip.Content>Salin subtask ini ke Todo A</Tooltip.Content>
											</Tooltip.Root>
										</div>
									{/each}
								</div>
							{:else}
								<span class="text-muted-foreground italic text-[11px]">(Tidak ada subtask)</span>
							{/if}
						</div>
					</div>
				</div>

				<!-- Waktu & Riwayat -->
				<div class="space-y-1">
					<span class="text-[10px] font-bold text-muted-foreground uppercase tracking-wider block">Waktu & Riwayat Aktivitas</span>
					<div class={cn('gap-2', layout === 'split' ? 'grid grid-cols-2' : 'space-y-1.5')}>
						<!-- A -->
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 space-y-1 min-w-0">
							<span class="text-[9px] text-muted-foreground font-bold block">Todo A</span>
							<div class="text-[10px] text-muted-foreground space-y-0.5">
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Dibuat:</span> {taskA.created_at ? formatFullDate(taskA.created_at) : '-'}</p>
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Status update:</span> {taskA.status_updated_at ? formatFullDate(taskA.status_updated_at) : '-'}</p>
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Riwayat event:</span> {eventsA.length} kali perubahan</p>
							</div>
						</div>
						<!-- B -->
						<div class="p-2 rounded-md bg-muted/40 border border-border/60 space-y-1 min-w-0">
							<span class="text-[9px] text-muted-foreground font-bold block">Todo B</span>
							<div class="text-[10px] text-muted-foreground space-y-0.5">
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Dibuat:</span> {taskB.created_at ? formatFullDate(taskB.created_at) : '-'}</p>
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Status update:</span> {taskB.status_updated_at ? formatFullDate(taskB.status_updated_at) : '-'}</p>
								<p class="break-words [overflow-wrap:anywhere]"><span class="font-medium text-foreground">Riwayat event:</span> {eventsB.length} kali perubahan</p>
							</div>
						</div>
					</div>
				</div>
			</div>

		{:else}
			<div class="py-12 text-center text-xs text-muted-foreground">
				Pilih dua todo untuk melihat perbandingannya.
			</div>
		{/if}

		<Dialog.Footer class="pt-2 border-t border-border shrink-0 flex-wrap">
			<Button
				variant="outline"
				size="sm"
				class="h-8 text-xs"
				onclick={() => (open = false)}
			>
				Tutup
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
