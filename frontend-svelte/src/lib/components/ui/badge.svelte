<script lang="ts">
	import { cn } from '#lib/utils';
	import type { Snippet } from 'svelte';

	let {
		class: klass = '',
		show = false,
		children
	}: { class?: string; show?: boolean; children?: Snippet } = $props();
</script>

{#if show}
	<span
		class={cn(
			'ml-1 inline-block rounded-full bg-slate-300/60 px-1.5 text-[10px] text-slate-600',
			klass
		)}
	>
		{@render children?.()}
	</span>
{/if}
