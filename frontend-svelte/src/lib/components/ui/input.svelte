<script lang="ts">
	import { cn } from '#lib/utils';
	import type { HTMLInputAttributes } from 'svelte/elements';

	let { class: klass = '', value = $bindable(), ...rest }: HTMLInputAttributes = $props();
</script>

<input
	bind:value
	class={cn(
		'flex w-full min-w-0 rounded-md border bg-card px-2 py-1.5 text-xs outline-none',
		'placeholder:text-muted-foreground focus-visible:border-primary focus-visible:ring-3 focus-visible:ring-primary/15',
		klass
	)}
	{...rest}
/>
