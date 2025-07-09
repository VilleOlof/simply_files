<script lang="ts">
	import { GITHUB_URL } from '$lib';
	import { Toaster } from 'svelte-french-toast';
	import '../app.css';
	import { onMount } from 'svelte';

	let { children } = $props();

	let is_mobile = $state(false);
	const handleResize = () => {
		is_mobile = screen.orientation.type.startsWith('portrait') || window.innerWidth < 640;
	};

	onMount(() => {
		handleResize();

		window.addEventListener('resize', handleResize);

		return () => {
			window.removeEventListener('resize', handleResize);
		};
	});
</script>

<svelte:head>
	<title>Simply... Files</title>
</svelte:head>

<main
	class="font-viga bg-background text-text flex h-dvh w-dvw flex-col items-center overflow-y-auto pb-12"
>
	<div class="mb-4 mt-6 flex items-center gap-2 sm:mb-8 sm:mt-8">
		<a href="/" aria-label="home" class="transition-opacity hover:opacity-80">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.25"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="text-primary drop-shadow-secondary w-9 drop-shadow sm:w-14 lg:w-20"
				><path
					d="M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z"
				/><path d="M12 22V12" /><polyline points="3.29 7 12 12 20.71 7" /><path
					d="m7.5 4.27 9 5.15"
				/></svg
			>
		</a>
		<p class="text-center text-2xl font-extrabold sm:text-4xl lg:text-5xl">Simply... Files</p>
	</div>

	{@render children()}

	<p class=" absolute bottom-0 right-0 m-3">
		<a href={GITHUB_URL} class="hover:underline">Github</a> | 1.0.0
	</p>

	<Toaster
		position={is_mobile ? 'bottom-left' : 'bottom-left'}
		toastOptions={{
			duration: 8000,
			style:
				'background: oklch(0.2745 0.016 248.31); color: oklch(86.26% 0.014 262.38); filter: drop-shadow(0.3rem 0.3rem 0px #0e141a); border-radius: 0.25rem'
		}}
	/>
</main>
