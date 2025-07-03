<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { PUBLIC_BACKEND } from '$env/static/public';
	import { change_access, format_path, type FileMetadata } from '$lib';
	import prettyBytes from 'pretty-bytes';

	const { file }: { file: FileMetadata } = $props();
	const date = new Date(file.modified * 1000);
	let stop_top_level_click = false;
	console.log(file);

	function get_download_link(file: FileMetadata): string {
		return `${PUBLIC_BACKEND}/d/${file.id}`;
	}

	async function delete_thing(file: FileMetadata) {
		stop_top_level_click = true;
		dispatchEvent(new CustomEvent('custom-delete-thing', { detail: { file } }));
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onclick={async () => {
		if (stop_top_level_click) {
			stop_top_level_click = false;
			return;
		}

		if (file.is_dir) {
			// append to the current location if the file is a directory
			await goto(window.location.href + `/${file.path}`);
		}
	}}
	class:cursor-pointer={file.is_dir}
	class:hover:bg-background-3={file.is_dir}
	class:select-none={file.is_dir}
	class="bg-background-2 drop-shadow-box drop-shadow-background-3 flex justify-between gap-8 px-3 py-1 transition-colors"
>
	<div class="flex gap-0.5">
		<button
			onclick={() => delete_thing(file)}
			aria-label="Delete"
			title="Delete"
			class="hover:bg-background-1 z-10 cursor-pointer select-all rounded px-1 transition-colors"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="text-text-2 w-5"
				><path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path
					d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"
				/></svg
			>
		</button>

		{#if !file.is_dir}
			<div class="flex gap-0.5">
				<button
					onclick={() => {
						const link = get_download_link(file);
						const a = document.createElement('a');
						a.href = link;
						document.body.appendChild(a);
						a.click();
						document.body.removeChild(a);
					}}
					aria-label="Download"
					title="Download"
					class="hover:bg-background-1 cursor-pointer rounded px-1 transition-colors"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="text-text-2 w-5"
						><path d="M12 15V3" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path
							d="m7 10 5 5 5-5"
						/></svg
					>
				</button>

				<button
					onclick={async () => {
						const link = get_download_link(file);
						// force enable public access
						await change_access(file, 1);
						await invalidateAll();

						navigator.clipboard.writeText(link);
					}}
					aria-label="Copy download link"
					title="Copy download link"
					class="hover:bg-background-1 cursor-pointer rounded px-1 transition-colors"
					><svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="text-text-2 w-5"
						><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path
							d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
						/></svg
					></button
				>

				<button
					onclick={async () => {
						await change_access(file, file.access == 1 ? 0 : 1);
						await invalidateAll();
					}}
					aria-label="Change Access"
					title="Change Access"
					class="hover:bg-background-1 cursor-pointer rounded px-1 transition-colors"
				>
					{#if file.access == 1}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="text-text-2 w-5"
							><path
								d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"
							/><circle cx="12" cy="12" r="3" /></svg
						>
					{:else}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="text-text-2 w-5"
							><path
								d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49"
							/><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" /><path
								d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143"
							/><path d="m2 2 20 20" /></svg
						>
					{/if}
				</button>
			</div>
		{:else}
			<div class="w-[5.5rem]"></div>
		{/if}

		<p class:underline={file.is_dir}>{format_path(file.path)}</p>
	</div>

	<div class="flex gap-3">
		{#if !file.is_dir}
			<p>{prettyBytes(file.size)}</p>

			<p>{date.toLocaleDateString()}</p>
		{/if}
	</div>
</div>
