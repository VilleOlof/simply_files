<script lang="ts">
	import { onMount } from 'svelte';
	import { download, upload } from './speed_test';
	import { browser } from '$app/environment';

	let down = $state(0);
	let up = $state(0);
	let is_measuring = $state(false);
	let time_active = $state(0);

	async function measure() {
		try {
			is_measuring = true;
			down = await download();
			up = await upload();
			is_measuring = false;
		} catch (error) {
			console.error('Error during speed test:', error);
			is_measuring = false;
		}
	}

	let interval: ReturnType<typeof setInterval> | null = null;
	$effect(() => {
		if (is_measuring) {
			time_active = 0;
			interval = setInterval(() => {
				time_active += 0.1;
			}, 100);
		} else {
			if (interval) clearInterval(interval);
		}
	});

	onMount(() => {
		if (!browser) return;
		const latest_download = localStorage.getItem('latest_download_speed');
		const latest_upload = localStorage.getItem('latest_upload_speed');

		if (latest_download) down = parseFloat(latest_download);
		if (latest_upload) up = parseFloat(latest_upload);
	});
</script>

<div class=" absolute right-0 top-0 m-8 flex flex-col items-end justify-end gap-3">
	<button
		title="Takes up about 200 MB of bandwidth"
		class="bg-background-2 hover:bg-background-3 drop-shadow-background-3 drop-shadow-box w-full cursor-pointer transition-colors"
		onclick={measure}
		>{is_measuring ? `Measuring... (${time_active.toFixed(1)}s)` : 'Measure'}</button
	>
	<div class="bg-background-2 drop-shadow-background-3 drop-shadow-box flex flex-col px-4 py-1">
		<div class="flex w-full flex-row justify-between gap-2">
			<span>Download:</span>
			<span>{down.toFixed(0)} Mbps</span>
		</div>
		<div class="flex w-full flex-row justify-between gap-2">
			<span>Upload:</span>
			<span>{up.toFixed(0)} Mbps</span>
		</div>
	</div>
</div>
