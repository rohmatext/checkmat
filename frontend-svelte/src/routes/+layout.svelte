<script lang="ts">
	import '../app.css';
	import * as Tooltip from '#lib/components/ui/tooltip/index.js';

	/** @type {{children: import('svelte').Snippet}} */
	let { children } = $props();
</script>

<Tooltip.Provider delayDuration={300}>
	{@render children()}
</Tooltip.Provider>
