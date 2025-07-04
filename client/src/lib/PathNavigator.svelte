<script lang="ts">
	import { afterNavigate, goto, invalidateAll } from '$app/navigation';
	import { onMount, tick } from 'svelte';
	import NewDirectory from './NewDirectory.svelte';

	const { path }: { path: string } = $props();

	let path_parts = $state(path.split('/').filter((part) => part.length > 0));

	let pathContainer: HTMLDivElement;
	let new_directory_dialog = $state(false);

	onMount(() => {
		scroll_path_container();
	});

	function scroll_path_container() {
		if (pathContainer) pathContainer.scrollLeft = pathContainer.scrollWidth;
	}

	function update_parts() {
		// special hidden edge case for one-time uploads
		if (path === '.public_uploads') {
			path_parts = ['.public_uploads'];
			return;
		}

		const parts = location.pathname.split('/').filter((part) => part.length > 0);
		if (['m', '/m', '/m/'].includes(parts[0])) {
			parts.shift(); // Remove 'm' or empty part at the start
		}

		path_parts = parts;
	}

	afterNavigate(() => {
		update_parts();

		// wait until next tick to ensure DOM is updated
		tick().then(() => {
			scroll_path_container();
		});
	});
</script>

<div class="mb-5 flex w-2/3 gap-4 xl:w-1/3">
	<button
		onclick={async () => {
			const parentPath = path_parts.slice(0, -1).join('/');
			await goto(`/m/${parentPath}`);
		}}
		aria-label="Go to parent directory"
		title="Go to parent directory"
		class="bg-background-2 hover:bg-background-3 drop-shadow-box drop-shadow-background-3 flex cursor-pointer items-center gap-2 rounded p-1 transition-colors"
		><svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="w-8"><path d="m18 15-6-6-6 6" /></svg
		></button
	>

	<button
		onclick={() => invalidateAll()}
		aria-label="Reload current directory"
		title="Reload current directory"
		class="bg-background-2 hover:bg-background-3 drop-shadow-box drop-shadow-background-3 flex cursor-pointer items-center gap-2 rounded p-1 transition-colors"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="w-8 p-0.5"
			><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /></svg
		>
	</button>

	<div
		class="bg-background-2 drop-shadow-box drop-shadow-background-3 flex min-w-0 flex-1 items-center gap-2 rounded px-3 py-1 text-xl"
	>
		<!-- always at the start -->
		<button
			aria-label="Go to root directory"
			title="Go to root directory"
			onclick={async () => await goto('/m')}
			class="hover:bg-background-1 flex-shrink-0 cursor-pointer rounded p-1 transition-colors"
			><svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="w-6"
				><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8" /><path
					d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
				/></svg
			></button
		>

		<div bind:this={pathContainer} class="flex min-w-0 flex-1 items-center gap-2 overflow-x-hidden">
			{#each path_parts as part, index}
				<p class="text-text-1 flex-shrink-0 select-none">/</p>
				<button
					class="hover:bg-background-1 flex-shrink-0 cursor-pointer whitespace-nowrap rounded px-0.5 transition-colors"
					onclick={async () => {
						const newPath = path_parts.slice(0, index + 1).join('/');
						await goto(`/m/${newPath}`);
					}}>{part}</button
				>
			{/each}
		</div>

		<!-- always at the end-->
		<div class="text-text-1 flex flex-shrink-0 items-center gap-2">
			<button
				onclick={() => (new_directory_dialog = true)}
				title="Add new directory"
				class="hover:bg-background-1 cursor-pointer rounded px-2 transition-colors">{'+'}</button
			>
		</div>
	</div>
</div>

<NewDirectory bind:open={new_directory_dialog} />
