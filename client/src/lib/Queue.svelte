<script lang="ts">
	import { onMount } from 'svelte';
	import { file_complete_handler, type QueueChanged } from './queue';
	import prettyBytes from 'pretty-bytes';

	let queue_changed = $state<QueueChanged | null>(null);

	function handle_queue_change(e: Event) {
		const data: QueueChanged = (e as CustomEvent).detail;
		queue_changed = data;
		console.log('Queue changed:', data);
	}

	onMount(() => {
		addEventListener('queue-changed', handle_queue_change);
		addEventListener('queue-next', file_complete_handler);

		return () => {
			removeEventListener('queue-changed', handle_queue_change);
			removeEventListener('queue-next', file_complete_handler);
		};
	});
</script>

{#if queue_changed && queue_changed.current_item}
	{@const curr = queue_changed.current_item.file}

	<div
		class="bg-background-2 drop-shadow-box drop-shadow-background-3 absolute bottom-0 right-0 mb-12 mr-4 flex max-w-52 flex-col gap-2 overflow-hidden truncate rounded p-3"
	>
		<div class="flex gap-3">
			<div class="flex w-full items-center justify-between gap-2">
				<p class="truncate">{curr.name}</p>
				<p>{prettyBytes(curr.size)}</p>
			</div>
		</div>

		{#if queue_changed.queue_length > 1}
			<div class="bg-primary/60 h-1 w-full rounded"></div>

			<div class="text-text-2 flex flex-col gap-3">
				{#each queue_changed?.queue as item}
					<div class="flex items-center justify-between gap-2">
						<p class="truncate">{item.name}</p>
						<p>{prettyBytes(item.size)}</p>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}
