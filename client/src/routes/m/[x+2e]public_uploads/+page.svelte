<script lang="ts">
	import FileHandler from '$lib/FileHandler.svelte';
	import type { PageProps } from './$types';
	import TopStatusBar from '$lib/TopStatusBar.svelte';
	import FileList from '$lib/FileList.svelte';
	import { invalidateAll } from '$app/navigation';
	import { fuckery_rust_time_to_date } from '$lib/format';
	import { delete_link, copy_link } from '$lib/link';
	import { notification } from '$lib/toast';
	import QrCode from '$lib/QRCode.svelte';

	const { data }: PageProps = $props();

	let qr_open = $state(false);
	let qr_link_id = $state('');
</script>

<TopStatusBar file_system={data.file_system} storage_limit={data.storage_limit} />

<FileList files={data.files} path={'.public_uploads'} />

<div class="my-1.5"></div>

<div class="flex w-11/12 flex-col gap-2 md:w-2/3 xl:w-1/3">
	{#each data.unused_links as link}
		<div
			class="bg-background-2 drop-shadow-box drop-shadow-background-3 flex justify-between gap-8 rounded px-3 py-1 transition-colors"
		>
			<div class="flex gap-2 md:gap-0.5">
				<div class="flex gap-0.5">
					<button
						onclick={async () => {
							await delete_link(link.id);
							await invalidateAll();
						}}
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

					<button
						onclick={() => {
							qr_link_id = link.id;
							qr_open = true;
						}}
						class="hover:bg-background-1 cursor-pointer rounded px-1 transition-colors"
						aria-label="Show QR Code"
						title="Show QR Code"
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
							><rect width="5" height="5" x="3" y="3" rx="1" /><rect
								width="5"
								height="5"
								x="16"
								y="3"
								rx="1"
							/><rect width="5" height="5" x="3" y="16" rx="1" /><path
								d="M21 16h-3a2 2 0 0 0-2 2v3"
							/><path d="M21 21v.01" /><path d="M12 7v3a2 2 0 0 1-2 2H7" /><path
								d="M3 12h.01"
							/><path d="M12 3h.01" /><path d="M12 16v.01" /><path d="M16 12h1" /><path
								d="M21 12v.01"
							/><path d="M12 21v-1" /></svg
						>
					</button>

					<button
						onclick={async () => {
							copy_link(link);
							notification.success('Copied link to clipboard');
						}}
						aria-label="Copy link"
						title="Copy link"
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

					<div class="w-[1rem] md:w-[1.75rem]"></div>

					<p>{link.id}</p>
				</div>
			</div>

			<p class="hidden md:flex">{fuckery_rust_time_to_date(link.created_at).toLocaleString()}</p>
		</div>
	{/each}

	{#if data.unused_links.length === 0}
		<p class="text-text-2 ml-4 italic">No unused links.</p>
	{/if}
</div>

<FileHandler endpoint={'/m/upload'} />

<QrCode bind:open={qr_open} link_id={qr_link_id} />
