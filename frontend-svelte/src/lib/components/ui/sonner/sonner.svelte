<script lang="ts">
	import { Toaster as Sonner, type ToasterProps } from 'svelte-sonner';

	let { closeButton = true, ...props }: ToasterProps = $props();
</script>

<Sonner {closeButton} {...props} />
