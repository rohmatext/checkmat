<script>
	import { Select as SelectPrimitive } from "bits-ui";
	import { HugeiconsIcon } from "@hugeicons/svelte"
	import { ArrowDown01Icon } from '@hugeicons/core-free-icons';
	import { cn } from "#lib/utils";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	} = $props();
</script>

<SelectPrimitive.ScrollDownButton
	bind:ref
	data-slot="select-scroll-down-button"
	class={cn("bg-popover z-10 flex cursor-default items-center justify-center py-1 [&_svg:not([class*='size-'])]:size-4", className)}
	{...restProps}
>
	<HugeiconsIcon icon={ArrowDown01Icon} strokeWidth={2}  />
</SelectPrimitive.ScrollDownButton>