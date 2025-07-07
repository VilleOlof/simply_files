<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import prettyBytes from 'pretty-bytes';
	import { notification } from './toast';
	import {
		type FileMetadata,
		rename_file,
		get_download_link,
		change_access,
		get_preview_link
	} from './file';
	import { format_path } from './format';
	import { onMount } from 'svelte';

	const { file }: { file: FileMetadata } = $props();
	const date = new Date(file.modified * 1000);
	let stop_top_level_click = false;

	let debounce_timeout: ReturnType<typeof setTimeout> | null = null;
	async function handle_rename(event: Event) {
		const input = event.target as HTMLInputElement;
		const new_path = input.value.trim();

		let current_name = file.path.split('/').pop() || '';
		if (current_name == new_path) return;
		if (!new_path || new_path.length === 0) return;
		if (new_path.includes('/') || new_path.includes('\\')) return;

		if (debounce_timeout) {
			clearTimeout(debounce_timeout);
		}

		debounce_timeout = setTimeout(async () => {
			try {
				await rename_file(file, new_path);
				await invalidateAll();
			} catch (error) {
				console.error('Failed to rename file:', error);
			}
		}, 500);
	}

	async function delete_thing(file: FileMetadata) {
		stop_top_level_click = true;
		dispatchEvent(new CustomEvent('custom-delete-thing', { detail: { file } }));
	}

	let shift_pressed = $state(false);
	function handle_shift(event: KeyboardEvent) {
		if (event.shiftKey) {
			shift_pressed = true;
		} else {
			shift_pressed = false;
		}
	}

	onMount(() => {
		document.addEventListener('keydown', handle_shift);
		document.addEventListener('keyup', handle_shift);
		return () => {
			document.removeEventListener('keydown', handle_shift);
			document.removeEventListener('keyup', handle_shift);

			if (debounce_timeout) {
				clearTimeout(debounce_timeout);
			}
		};
	});
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
	class="bg-background-2 drop-shadow-box drop-shadow-background-3 flex justify-between gap-8 rounded px-3 py-1 transition-colors"
>
	<div class="flex flex-col-reverse gap-2 md:w-3/5 md:flex-row md:gap-0.5">
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
							const link = get_download_link(file.id);
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
							const link = shift_pressed ? get_download_link(file.id) : get_preview_link(file.id);
							// force enable public access
							if (file.access == 0) {
								await change_access(file, 1);
								await invalidateAll();
							}

							navigator.clipboard.writeText(link);
							notification.success(
								`Copied ${shift_pressed ? 'download' : 'preview'} link to clipboard`
							);
						}}
						aria-label="Copy {shift_pressed ? 'download' : 'preview'} link"
						title="Copy {shift_pressed ? 'download' : 'preview'} link"
						class="hover:bg-background-1 cursor-pointer rounded px-1 transition-colors"
						><svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class:text-text={shift_pressed}
							class:text-text-2={!shift_pressed}
							class=" w-5"
							><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path
								d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
							/></svg
						></button
					>

					<button
						onclick={async () => {
							await goto(`/d/${file.id}`);
						}}
						aria-label="Open file preview"
						title="Open file preview"
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
							><path d="M21 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6" /><path
								d="m21 3-9 9"
							/><path d="M15 3h6v6" /></svg
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
		</div>

		{#if file.is_dir}
			<p class="underline">{format_path(file.path)}</p>
		{:else}
			<input
				type="text"
				value={format_path(file.path)}
				onchange={handle_rename}
				class=" focus:bg-background-1 w-full rounded outline-none"
			/>
		{/if}
	</div>

	<div class="flex gap-3">
		{#if !file.is_dir}
			<p>{prettyBytes(file.size)}</p>

			<p class="hidden md:flex">{date.toLocaleDateString()}</p>
		{/if}
	</div>
</div>
