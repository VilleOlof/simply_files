<script lang="ts">
	import { GITHUB_URL } from '$lib';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>Simply... Files</title>
</svelte:head>

<main class="font-viga bg-background text-text flex h-dvh w-dvw flex-col items-center">
	<div class="my-8 flex items-center gap-2">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.25"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="text-primary drop-shadow-secondary w-20 drop-shadow"
			><path
				d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"
			/><path d="M12 22V12" /><polyline points="3.29 7 12 12 20.71 7" /><path
				d="m7.5 4.27 9 5.15"
			/></svg
		>
		<h1>Simply... Files</h1>
	</div>

	{@render children()}

	<p class=" absolute bottom-0 right-0 m-3">
		<a href={GITHUB_URL} class="hover:underline">Github</a> | 0.0.0
	</p>
</main>
