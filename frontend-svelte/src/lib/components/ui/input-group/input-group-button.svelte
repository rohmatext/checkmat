<script module>
	import { tv } from "tailwind-variants";

	const inputGroupButtonVariants = tv({
		base: "gap-2 rounded-4xl text-sm flex items-center shadow-none",
		variants: {
			size: {
				xs: "h-6 gap-1 px-1.5 [&>svg:not([class*='size-'])]:size-3.5",
				sm: "",
				"icon-xs": "size-6 p-0 has-[>svg]:p-0",
				"icon-sm": "size-8 p-0 has-[>svg]:p-0",
			},
		},
		defaultVariants: {
			size: "xs",
		},
	});
</script>

<script>
	import { Button } from "#lib/components/ui/button/index.js";
	import { cn } from "#lib/utils.js";
	let {
		ref = $bindable(null),
		class: className,
		children,
		type = "button",
		variant = "ghost",
		size = "xs",
		...restProps
	} = $props();
</script>

<Button
	bind:ref
	{type}
	data-size={size}
	{variant}
	class={cn(inputGroupButtonVariants({ size }), className)}
	{...restProps}
>
	{@render children?.()}
</Button>