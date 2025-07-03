<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { add_directory, clean_path } from '$lib';
	import { onMount } from 'svelte';
	import Popup from './Popup.svelte';

	let { open = $bindable() }: { open: boolean } = $props();

	let directory = $state('');
	let input_element: HTMLInputElement | null = $state(null);

	$effect(() => {
		if (open && input_element) {
			input_element.focus();
		}
	});

	async function create() {
		if (directory.trim() === '') return;

		open = false;

		const dir = directory.trim();
		let full_path = clean_path(window.location.pathname);
		full_path =
			full_path + (full_path.endsWith('/') ? '' : '/') + (dir.startsWith('/') ? dir.slice(1) : dir);
		if (full_path.startsWith('/')) full_path = full_path.slice(1); // remove leading slash if exists

		await add_directory(full_path);
		directory = '';

		await invalidateAll();
	}

	onMount(() => {
		const handleKeydown = (event: KeyboardEvent) => {
			if (!open) return;
			if (event.key === 'Enter') {
				create();
			}
		};

		window.addEventListener('keydown', handleKeydown);

		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<Popup bind:open>
	<div class="bg-background-3 rounded p-4">
		<h2>Add new directory</h2>
		<p class="text-text-1">This will be created based of your current open directory</p>

		<input
			type="text"
			bind:value={directory}
			bind:this={input_element}
			placeholder="Directory name"
			class="bg-background-1 text-text-1 mt-4 w-full rounded p-2 outline-none"
		/>

		<div class="mt-4 flex justify-end gap-2">
			<button
				onclick={() => (open = false)}
				class="bg-background-1 hover:bg-background-2 cursor-pointer rounded px-4 py-2 transition-colors"
			>
				Cancel
			</button>
			<button
				onclick={create}
				class="bg-background-1 hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-4 py-2 transition-all"
			>
				Create
			</button>
		</div>
	</div>
</Popup>
