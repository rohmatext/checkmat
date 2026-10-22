<script lang="ts">
	import '../app.css';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';
	import { onMount } from 'svelte';

	/** @type {{children: import('svelte').Snippet}} */
	let { children } = $props();

	onMount(() => {
		const handleContextMenu = (e: MouseEvent) => {
			// Cegah menu klik kanan default browser (refresh, save, print, dll)
			e.preventDefault();
		};
		window.addEventListener('contextmenu', handleContextMenu);
		return () => {
			window.removeEventListener('contextmenu', handleContextMenu);
		};
	});
</script>

<Tooltip.Provider delayDuration={300} ignoreNonKeyboardFocus={true} disableHoverableContent={true}>
	{@render children()}
</Tooltip.Provider>
