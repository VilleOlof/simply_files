<script lang="ts">
	import { onMount } from 'svelte';
	import { slide } from 'svelte/transition';
	import { type UploadEndpoint, upload_file } from './file';
	import { clean_path } from './format';
	import prettyBytes from 'pretty-bytes';
	import type { UploadFile } from './upload';
	import { goto, invalidateAll } from '$app/navigation';
	import { notification } from './toast';

	const { endpoint, one_time_id }: { endpoint: UploadEndpoint; one_time_id?: string } = $props();

	type ProgressData = {
		percent: number;
		speed: number;
		chunk_index: number;
		total_chunks: number;
		estimated_time: string;
	};
	let data = $state<ProgressData | null>(null);

	let upload_progress = $state<number | null>(null);
	let current_speed = $state<number | null>(null);
	let chunk_info = $state<{ chunk_index: number; total_chunks: number } | null>(null);

	async function file_upload_complete(e: Event) {
		data = null;

		removeEventListener('upload-complete', file_upload_complete);
		removeEventListener('upload-progress', file_upload_progress);

		notification.success('Upload successful!');
		const details: UploadFile.UploadFileComplete = (e as CustomEvent).detail;
		if (details.link_upload) {
			// preview the file if it's a link upload
			await goto(`/d/${details.db_file.id}`, { invalidateAll: true });
		} else {
			await invalidateAll();
		}
	}

	function file_upload_progress(e: Event) {
		const details: UploadFile.UploadFileEventDetail = (e as CustomEvent).detail;

		data = {
			percent: details.percent,
			speed: calculate_speed(details),
			chunk_index: details.chunk_index,
			total_chunks: details.total_chunks,
			estimated_time: calculate_estimated_time(details)
		};
	}

	function calculate_speed(details: UploadFile.UploadFileEventDetail): number {
		return Math.round(details.bytes_sent / ((Date.now() - details.upload_start_time) / 1000));
	}

	function calculate_estimated_time(details: UploadFile.UploadFileEventDetail): string {
		const speed = calculate_speed(details);
		if (speed === 0) return '00';
		const remaining_bytes = details.total_bytes - details.bytes_sent;
		const time_left = remaining_bytes / speed;

		// HH:MM:SS format
		const hours = Math.floor(time_left / 3600);
		const minutes = Math.floor((time_left % 3600) / 60);
		const seconds = Math.floor(time_left % 60);

		const formatted_hours = hours > 0 ? `${String(hours).padStart(2, '0')}:` : '';
		const formatted_minutes = minutes > 0 ? `${String(minutes).padStart(2, '0')}:` : '';
		const formatted_seconds = String(seconds).padStart(2, '0');
		return `${formatted_hours}${formatted_minutes}${formatted_seconds}`;
	}

	async function upload(files: FileList) {
		if (files !== null && files.length > 0) {
			addEventListener('upload-complete', file_upload_complete);
			addEventListener('upload-progress', file_upload_progress);

			for (let i = 0; i < files.length; i++) {
				const file = files.item(i);
				if (file === null) continue; // skip if file is null

				let path = clean_path(window.location.pathname);
				path = path + (path.endsWith('/') ? '' : '/') + file.name;
				if (path.startsWith('/')) path = path.slice(1); // remove leading slash if exists

				upload_file(
					file,
					endpoint,
					// mhmhm beautiful
					one_time_id !== undefined ? file.name : path,
					one_time_id !== undefined ? `id=${one_time_id}` : undefined
				);
			}
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

<!-- so like if we upload multiple files, this upload_progress will jump
between all of the current parallel uploads, making the bar go up or down anytime
so a solution to this would be nice but not until someone complains <3
-->
{#if data !== null}
	<div
		class="bg-background-2 absolute left-0 top-0 flex w-full items-center justify-center py-1 md:h-6"
		transition:slide={{ axis: 'y' }}
	>
		<div
			class="bg-primary absolute left-0 top-0 h-full transition-all duration-300"
			style="width: {data.percent}%"
		></div>

		<span class="text-text-1 bg-background-2/60 font-source z-10 h-full px-4 text-sm">
			{prettyBytes(data.speed)}/s ({data.chunk_index}/{data.total_chunks}) {data.estimated_time} remaining
		</span>
	</div>
{/if}

<svelte:body on:drop={drop_file} on:dragover={drop_over} />
