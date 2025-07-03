<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { clean_path, upload_file, type UploadEndpoint } from '$lib';
	import { onDestroy, onMount } from 'svelte';
	import { slide } from 'svelte/transition';

	const { endpoint, one_time_id }: { endpoint: UploadEndpoint; one_time_id?: string } = $props();

	let upload_progress = $state<number | null>(null);

	function file_upload_complete() {
		upload_progress = null;

		removeEventListener('upload-complete', file_upload_complete);
		removeEventListener('upload-progress', file_upload_progress);
	}

	function file_upload_progress(e: Event) {
		const { percent }: { percent: number } = (e as CustomEvent).detail;

		upload_progress = percent;
	}

	async function upload(files: FileList) {
		if (files !== null && files.length > 0) {
			addEventListener('upload-complete', file_upload_complete);
			addEventListener('upload-progress', file_upload_progress);

			let path = clean_path(window.location.pathname);
			path = path + (path.endsWith('/') ? '' : '/') + files[0].name;
			if (path.startsWith('/')) path = path.slice(1); // remove leading slash if exists
			console.log('uploading to', path);

			upload_file(
				files[0],
				endpoint,
				// mhmhm beautiful
				one_time_id !== undefined ? files[0].name + `?id=${one_time_id}` : path
			);

			await invalidateAll();
		}
	}

	async function drop_file(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
		if (!event.dataTransfer) return;

		let files = event.dataTransfer.files;

		if (event.dataTransfer.items[0].kind !== 'file') return;

		await upload(files);
	}
	function drop_over(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
	}

	async function manual_upload(event: Event) {
		const { files }: { files: FileList } = (event as CustomEvent).detail;

		if (files !== null && files.length > 0) {
			await upload(files);
		}
	}

	onMount(() => {
		addEventListener('manual-upload', manual_upload);

		return () => {
			removeEventListener('manual-upload', manual_upload);
		};
	});
</script>

{#if upload_progress !== null}
	<div class="bg-background-2 absolute left-0 top-0 h-4 w-full" transition:slide={{ axis: 'y' }}>
		<div
			class="bg-primary h-full transition-all duration-300"
			style="width: {upload_progress}%"
		></div>
	</div>
{/if}

<svelte:body on:drop={drop_file} on:dragover={drop_over} />
