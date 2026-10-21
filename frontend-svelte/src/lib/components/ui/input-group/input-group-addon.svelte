<script module>
	import { tv } from "tailwind-variants";
	export const inputGroupAddonVariants = tv({
		base: "text-muted-foreground **:data-[slot=kbd]:bg-muted-foreground/10 h-auto gap-2 py-2 text-sm font-medium group-data-[disabled=true]/input-group:opacity-50 **:data-[slot=kbd]:rounded-4xl **:data-[slot=kbd]:px-1.5 [&>svg:not([class*='size-'])]:size-4 flex cursor-text items-center justify-center select-none",
		variants: {
			align: {
				"inline-start": "pl-3 has-[>button]:-ml-1 has-[>kbd]:ml-[-0.15rem] order-first",
				"inline-end": "pr-3 has-[>button]:-mr-1 has-[>kbd]:mr-[-0.15rem] order-last",
				"block-start": "px-3 pt-3 group-has-[>input]/input-group:pt-3 [.border-b]:pb-3 order-first w-full justify-start",
				"block-end": "px-3 pb-3 group-has-[>input]/input-group:pb-3 [.border-t]:pt-3 order-last w-full justify-start",
			},
		},
		defaultVariants: {
			align: "inline-start",
		},
	});
</script>

<script>
	import { cn } from "#lib/utils.js";
	let {
		ref = $bindable(null),
		class: className,
		children,
		align = "inline-start",
		...restProps
	} = $props();
</script>

<div
	bind:this={ref}
	role="group"
	data-slot="input-group-addon"
	data-align={align}
	class={cn(inputGroupAddonVariants({ align }), className)}
	onclick={(e) => {
		if ((e.target).closest("button")) {
			return;
		}
		e.currentTarget.parentElement?.querySelector("input")?.focus();
	}}
	{...restProps}
>
	{@render children?.()}
</div>