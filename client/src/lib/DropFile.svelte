<script lang="ts">
	import { upload_file, type UploadEndpoint } from '$lib';

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

	function upload(files: FileList) {
		if (files !== null && files.length > 0) {
			addEventListener('upload-complete', file_upload_complete);
			addEventListener('upload-progress', file_upload_progress);
			upload_file(
				files[0],
				endpoint,
				// mhmhm beautiful
				one_time_id !== undefined ? files[0].name + `?id=${one_time_id}` : files[0].name
			);
		}
	}

	async function drop_file(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
		if (!event.dataTransfer) return;

		let files = event.dataTransfer.files;

		if (event.dataTransfer.items[0].kind !== 'file') return;

		upload(files);
	}
	function drop_over(event: DragEvent) {
		event.preventDefault();
		event.stopPropagation();
	}
</script>

<svelte:body on:drop={drop_file} on:dragover={drop_over} />
