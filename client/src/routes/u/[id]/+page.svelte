<script lang="ts">
	import DropFile from '$lib/FileHandler.svelte';
	import { get_download_link, upload_button } from '$lib/file';
	import { notification } from '$lib/toast';
	import type { PageProps } from './$types';

	const { data }: PageProps = $props();
</script>

{#if !data.is_valid && !data.done}
	<div class="mt-12"></div>

	<div class="flex flex-col items-center justify-center gap-8">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="w-32 text-red-500"
			><path
				d="m18.84 12.25 1.72-1.71h-.02a5.004 5.004 0 0 0-.12-7.07 5.006 5.006 0 0 0-6.95 0l-1.72 1.71"
			/><path
				d="m5.17 11.75-1.71 1.71a5.004 5.004 0 0 0 .12 7.07 5.006 5.006 0 0 0 6.95 0l1.71-1.71"
			/><line x1="8" x2="8" y1="2" y2="5" /><line x1="2" x2="5" y1="8" y2="8" /><line
				x1="16"
				x2="16"
				y1="19"
				y2="22"
			/><line x1="19" x2="22" y1="16" y2="16" /></svg
		>

		<div class="flex flex-col items-center gap-2">
			<p class="text-balance text-center text-2xl">This is an invalid upload link...</p>
			<a href="/" class="hover:text-primary text-xl underline transition-colors">Go back</a>
		</div>
	</div>
{:else if data.done && data.file_id}
	<p class="text-xl">Sucessfully uploaded your file</p>

	<div class="my-2"></div>

	<div class="flex items-center gap-4">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="text-primary w-18"
			><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path
				d="M14 2v4a2 2 0 0 0 2 2h4"
			/><path d="m9 15 2 2 4-4" /></svg
		>

		<button
			class="bg-background-2 drop-shadow-background-3 drop-shadow-box hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-6 py-2 text-2xl transition-all"
			onclick={() => {
				const link = get_download_link(data.file_id);
				navigator.clipboard.writeText(link);
				notification.success('Copied link to clipboard');
			}}>Copy link</button
		>
	</div>
{:else}
	<p class="text-balance text-center">
		You've been given a link to where you can upload one file, one time!
	</p>
	<p class="text-balance text-center">
		After a successful upload, you can copy it's link and share it.
	</p>

	<div class="my-2"></div>

	<button
		onclick={() => upload_button(true)}
		class="bg-background-2 drop-shadow-background-3 drop-shadow-box hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-6 py-2 text-2xl transition-all"
		>Upload</button
	>

	<DropFile endpoint={'/o/upload'} one_time_id={data.id} />
{/if}
