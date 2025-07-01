<script lang="ts">
	import DropFile from '$lib/DropFile.svelte';
	import type { PageProps } from './$types';

	const { data }: PageProps = $props();
</script>

{#if !data.is_valid}
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
			<p class="text-2xl">This is an invalid upload link...</p>
			<a href="/" class="hover:text-primary text-xl underline transition-colors">Go back</a>
		</div>
	</div>
{:else}
	<p>upload</p>

	<DropFile endpoint={'/o/upload'} one_time_id={data.id} />
{/if}
