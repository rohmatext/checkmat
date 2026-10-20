<script lang="ts">
	import { cn } from '#lib/utils';
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		variant = 'default',
		class: klass = '',
		children,
		...rest
	}: HTMLButtonAttributes & {
		variant?: 'default' | 'ghost' | 'ok' | 'warn' | 'outline';
		children?: Snippet;
		class?: string;
	} = $props();

	const variants: Record<string, string> = {
		default: 'bg-primary text-primary-foreground hover:bg-primary/90',
		outline: 'border bg-card hover:bg-muted',
		ghost: 'hover:bg-muted',
		ok: 'bg-emerald-100 text-emerald-700 border-emerald-200 hover:bg-emerald-200/70',
		warn: 'bg-red-100 text-red-700 border-red-200 hover:bg-red-200/70'
	};
</script>

<button
	class={cn(
		'inline-flex cursor-pointer items-center justify-center gap-1 rounded-md border border-transparent px-2 py-1 text-xs font-medium whitespace-nowrap transition-colors disabled:pointer-events-none disabled:opacity-50',
		variants[variant],
		klass
	)}
	{...rest}
>
	{@render children?.()}
</button>
