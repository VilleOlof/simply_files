<script lang="ts">
	import { type FileMetadata } from '$lib';
	import { onMount } from 'svelte';
	import Delete from './Delete.svelte';
	import FileEntry from './FileEntry.svelte';
	import PathNavigator from './PathNavigator.svelte';

	const { files, path }: { files: FileMetadata[]; path: string } = $props();

	let {
		delete_dialiog_open,
		file
	}: {
		delete_dialiog_open: boolean;
		file: FileMetadata;
	} = $state({
		delete_dialiog_open: false,
		file: {} as FileMetadata
	});

	function handle_delete(event: Event) {
		const data = (event as CustomEvent).detail;
		console.log('handle_delete', data);
		if (data && data.file) {
			file = data.file;
			delete_dialiog_open = true;
		}
	}

	onMount(() => {
		addEventListener('custom-delete-thing', handle_delete);

		return () => {
			removeEventListener('custom-delete-thing', handle_delete);
		};
	});
</script>

<PathNavigator {path} />

<div class="flex w-2/3 flex-col gap-2 xl:w-1/3">
	{#each files as file}
		<FileEntry {file} />
	{/each}
</div>

<Delete bind:open={delete_dialiog_open} bind:file />
