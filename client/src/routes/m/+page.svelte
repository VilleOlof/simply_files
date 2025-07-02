<script lang="ts">
	import { format_path } from '$lib';
	import DropFile from '$lib/DropFile.svelte';
	import prettyBytes from 'pretty-bytes';
	import type { PageProps } from './$types';

	const { data }: PageProps = $props();
	let storage_percentage = $state((data.storage_limit.used / data.storage_limit.max) * 100);
	storage_percentage = 40;
</script>

<div
	class="bg-background-2 drop-shadow-box drop-shadow-background-3 mb-8 flex w-1/3 justify-between gap-3 px-8 py-2"
>
	<div class="flex items-center gap-2" title={data.file_system.about}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="lucide lucide-hard-drive-icon lucide-hard-drive"
			><line x1="22" x2="2" y1="12" y2="12" /><path
				d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"
			/><line x1="6" x2="6.01" y1="16" y2="16" /><line x1="10" x2="10.01" y1="16" y2="16" /></svg
		>
		<span class="text-text-1 hover:underline">{data.file_system.which}</span>
	</div>

	<div class="bg-background-1 relative h-6 w-fit min-w-[8rem] overflow-hidden rounded px-2">
		<div class="bg-primary absolute inset-0" style="width: {storage_percentage}%"></div>

		<div
			class="pointer-events-none relative z-10 flex h-full items-center justify-center text-sm font-semibold"
		>
			<!-- on green bar-->
			<span
				class="text-background-1 absolute inset-0 flex items-center justify-center"
				style="clip-path: inset(0 {100 - storage_percentage}% 0 0);"
			>
				{prettyBytes(data.storage_limit.used)} / {prettyBytes(data.storage_limit.max)}
			</span>

			<!-- outside green bar -->
			<span
				class="absolute inset-0 flex items-center justify-center text-white"
				style="clip-path: inset(0 0 0 {storage_percentage}%);"
			>
				{prettyBytes(data.storage_limit.used)} / {prettyBytes(data.storage_limit.max)}
			</span>
		</div>
	</div>

	<button
		class="bg-background-1 hover:bg-background-3 flex cursor-pointer items-center gap-2 rounded px-2 transition-colors"
	>
		<span>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="w-5"
				><path d="M12 3v12" /><path d="m17 8-5-5-5 5" /><path
					d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"
				/></svg
			>
		</span>
		Upload</button
	>
</div>

<div class="flex flex-col gap-1">
	{#each data.files as file}
		<div class="bg-background-2 flex justify-between gap-8 px-3 py-1">
			<p>{format_path(file.path)}</p>
			<p>{prettyBytes(file.size)}</p>
		</div>
	{/each}
</div>

<DropFile endpoint={'/m/upload'} />
