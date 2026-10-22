<script lang="ts">
	import { Copy, Check } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { cn } from '#lib/utils';

	let { text = '' }: { text: string } = $props();

	interface Segment {
		type: 'text' | 'copy';
		content: string;
	}

	function parseCopyTags(str: string): Segment[] {
		if (!str) return [];
		const regex = /\[copy\]([\s\S]*?)\[\/copy\]/gi;
		const segments: Segment[] = [];
		let lastIndex = 0;
		let match: RegExpExecArray | null;

		while ((match = regex.exec(str)) !== null) {
			if (match.index > lastIndex) {
				segments.push({
					type: 'text',
					content: str.slice(lastIndex, match.index)
				});
			}
			segments.push({
				type: 'copy',
				content: match[1]
			});
			lastIndex = regex.lastIndex;
		}

		if (lastIndex < str.length) {
			segments.push({
				type: 'text',
				content: str.slice(lastIndex)
			});
		}

		return segments;
	}

	const hasCopyTag = $derived(/\[copy\][\s\S]*?\[\/copy\]/i.test(text));
	const segments = $derived(hasCopyTag ? parseCopyTags(text) : []);
	let copiedIndexes = $state<Record<number, boolean>>({});

	async function handleCopy(content: string, index: number, e: MouseEvent | KeyboardEvent) {
		e.stopPropagation();
		e.preventDefault();
		try {
			await navigator.clipboard.writeText(content);
			copiedIndexes = { ...copiedIndexes, [index]: true };
			const preview = content.length > 25 ? content.slice(0, 22) + '...' : content;
			toast.success(`Disalin ke clipboard: ${preview}`);
			setTimeout(() => {
				copiedIndexes = { ...copiedIndexes, [index]: false };
			}, 1500);
		} catch {
			toast.error('Gagal menyalin ke clipboard');
		}
	}
</script>

{#if !hasCopyTag}
	{text}
{:else}
	{#each segments as seg, idx (idx)}
		{#if seg.type === 'text'}
			{seg.content}
		{:else}
			{@const isCopied = copiedIndexes[idx]}
			<!-- svelte-ignore a11y_interactive_supports_focus -->
			<span
				role="button"
				tabindex="0"
				onclick={(e) => handleCopy(seg.content, idx, e)}
				ondblclick={(e) => {
					e.stopPropagation();
					handleCopy(seg.content, idx, e);
				}}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						handleCopy(seg.content, idx, e);
					}
				}}
				class={cn(
					'inline-flex items-center gap-1 px-1.5 py-0.5 mx-0.5 rounded font-mono text-[11px] leading-tight select-all transition-all border align-baseline cursor-pointer group/copy',
					isCopied
						? 'bg-emerald-500/15 border-emerald-500/40 text-emerald-600 dark:text-emerald-400 font-semibold'
						: 'bg-muted/80 hover:bg-muted border-border/80 text-foreground hover:border-primary/50 shadow-2xs active:scale-95'
				)}
				title="Klik untuk salin ke clipboard"
			>
				{#if isCopied}
					<Check class="size-3 text-emerald-500 shrink-0" />
				{:else}
					<Copy class="size-3 text-muted-foreground/80 group-hover/copy:text-primary shrink-0 transition-colors" />
				{/if}
				<span class="break-all">{seg.content}</span>
			</span>
		{/if}
	{/each}
{/if}
