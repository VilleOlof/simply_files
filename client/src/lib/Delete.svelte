<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { PUBLIC_BACKEND } from '$env/static/public';
	import { get_good_path, type FileMetadata } from '$lib';
	import Popup from './Popup.svelte';

	let {
		open = $bindable(),
		file = $bindable()
	}: {
		open: boolean;
		file: FileMetadata;
	} = $props();

	let path = $state<string>('');
	$effect(() => {
		path = get_good_path(file);
	});

	async function delete_thing(file: FileMetadata) {
		let response: Response | undefined = undefined;
		if (file.is_dir) {
			response = await fetch(`${PUBLIC_BACKEND}/m/directory/${path}`, {
				method: 'DELETE',
				credentials: 'include'
			});
		} else {
			response = await fetch(`${PUBLIC_BACKEND}/m/delete_file/${path}`, {
				method: 'DELETE',
				credentials: 'include'
			});
		}

		if (response.ok) {
			await invalidateAll();
		} else {
			console.error('Failed to delete file:', response.status, response.statusText);
		}

		open = false;
	}
</script>

<Popup bind:open>
	<div class="bg-background-3 rounded p-4">
		<h2>Delete this {file.is_dir ? 'Directory' : 'File'}?</h2>
		<p class="text-text-1">
			This will permanently delete this {file.is_dir ? 'directory' : 'file'}
		</p>
		{#if file.is_dir}
			<p class="text-text-1">
				Deleting a directory only works if it <span class="text-text">no</span> entries
			</p>
		{/if}

		<p
			class="bg-background-1 mt-2 max-w-[34.75rem] overflow-scroll text-wrap break-all rounded px-3 py-2 text-xl text-red-500"
		>
			{path}
		</p>

		<div class="mt-4 flex justify-end gap-2">
			<button
				onclick={() => (open = false)}
				class="bg-background-1 hover:bg-background-2 cursor-pointer rounded px-4 py-2 transition-colors"
			>
				Cancel
			</button>
			<button
				onclick={() => delete_thing(file)}
				class="bg-background-1 hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-4 py-2 transition-all"
			>
				Delete
			</button>
		</div>
	</div>
</Popup>
