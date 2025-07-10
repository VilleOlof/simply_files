<script lang="ts">
	import FileHandler from '$lib/FileHandler.svelte';
	import Queue from '$lib/Queue.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<title>Simply... Files</title>
</svelte:head>

{@render children()}

<Queue />

<FileHandler endpoint={'/m/upload'} />
