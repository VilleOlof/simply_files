<script lang="ts">
	import { onMount } from 'svelte';
	import { file_complete_handler, type QueueChanged } from './queue';
	import prettyBytes from 'pretty-bytes';
	import { UploadFile } from './upload';
	import { calculate_estimated_time, calculate_speed } from './file';

	let queue_changed = $state<QueueChanged | null>(null);

	let queue_bytes_sent = $state(0);
	let queue_total_bytes = $state(0);
	let estimated_time = $state<string | null>(null);
	let latest_speeds: number[] = [];

	function handle_queue_change(e: Event) {
		const data: QueueChanged = (e as CustomEvent).detail;
		queue_changed = data;

		if (data.new_item) {
			queue_total_bytes += data.new_item.file.size;
		}

		if (queue_changed.current_item === null) {
			queue_bytes_sent = 0;
			queue_total_bytes = 0;
			estimated_time = null;
			console.log('Queue reset');
		}
	}

	function handle_upload_speed(e: Event) {
		const data: UploadFile.UploadFileEventDetail = (e as CustomEvent).detail;

		queue_bytes_sent += data.chunk_size;

		const speed = calculate_speed(data.bytes_sent, data.upload_start_time);
		latest_speeds.push(speed);
		if (latest_speeds.length > 10) {
			latest_speeds.shift(); // Keep the last 10 speeds
		}
		const average_speed = latest_speeds.reduce((a, b) => a + b, 0) / latest_speeds.length;

		estimated_time = calculate_estimated_time(queue_bytes_sent, queue_total_bytes, average_speed);
	}

	function file_complete(e: Event) {
		const data: UploadFile.UploadFileComplete = (e as CustomEvent).detail;
		// TODO: Improve this, since now it basically resets the estimation
		// and jumps from like 40 down to 0 and then counts back up to the real estimation
		// queue_total_bytes -= data.db_file.size;
		// queue_bytes_sent -= data.db_file.size;
	}

	onMount(() => {
		addEventListener('queue-changed', handle_queue_change);
		addEventListener('queue-next', file_complete_handler);
		addEventListener('upload-progress', handle_upload_speed);
		addEventListener('upload-complete', file_complete);

		return () => {
			removeEventListener('queue-changed', handle_queue_change);
			removeEventListener('queue-next', file_complete_handler);
			removeEventListener('upload-progress', handle_upload_speed);
			removeEventListener('upload-complete', file_complete);
		};
	});
</script>

{#if queue_changed && queue_changed.current_item}
	{@const curr = queue_changed.current_item.file}

	<div
		class="bg-background-3 drop-shadow-box drop-shadow-background-1 absolute bottom-0 right-0 mb-12 mr-4 flex max-w-52 flex-col gap-2 overflow-hidden truncate rounded p-3 shadow"
	>
		<div class="flex gap-3">
			<div class="flex w-full items-center justify-between gap-2">
				<p class="truncate">{curr.name}</p>
				<p>{prettyBytes(curr.size)}</p>
			</div>
		</div>

		{#if queue_changed.queue_length > 1}
			<div class="bg-primary/60 h-1 w-full rounded"></div>

			<div class="text-text-1 flex flex-col gap-3">
				{#each queue_changed?.queue.slice(1) as item}
					<div class="flex items-center justify-between gap-2">
						<p class="truncate">{item.name}</p>
						<p>{prettyBytes(item.size)}</p>
					</div>
				{/each}
			</div>
		{/if}

		{#if estimated_time}
			<p class="text-text-2 text-end text-sm">
				{estimated_time} remaining
			</p>
		{/if}
	</div>
{/if}
