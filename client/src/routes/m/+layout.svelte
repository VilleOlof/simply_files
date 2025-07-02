<script lang="ts">
	import SpeedTest from '$lib/SpeedTest.svelte';

	let { children } = $props();
</script>

{@render children()}
<!-- 
<SpeedTest /> -->
