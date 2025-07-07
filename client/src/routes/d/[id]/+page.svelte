<script lang="ts">
	import prettyBytes from 'pretty-bytes';
	import type { PageProps } from './$types';
	import { fuckery_rust_time_to_date } from '$lib/format';
	import { notification } from '$lib/toast';
	import QrCode from '$lib/QRCode.svelte';
	import { invalidateAll } from '$app/navigation';
	import { change_access_with_id, get_preview_link } from '$lib/file';
	import { browser } from '$app/environment';

	const { data }: PageProps = $props();

	let qr_dialog_open: boolean = $state(false);
	let qr_file_id: string | undefined = $state(undefined);
</script>

<svelte:head>
	<title>{data.meta.file_name}</title>
	<meta property="og:type" content="website" />
	<meta property="og:title" content={data.meta.file_name} />
	<meta property="og:description" content="Download, view or share" />
	{#if data.meta.mime_type.startsWith('video')}
		<meta property="og:video" content={data.url} />
		<meta property="og:video:type" content={data.meta.mime_type} />
	{:else if data.meta.mime_type.startsWith('image')}
		<meta property="og:image" content={data.url} />
		<meta property="og:image:type" content={data.meta.mime_type} />
	{:else if data.meta.mime_type.startsWith('audio')}
		<meta property="og:audio" content={data.url} />
		<meta property="og:audio:type" content={data.meta.mime_type} />
	{/if}
</svelte:head>

<div class="max-h-9/12 flex w-4/5 flex-col gap-4">
	<div class="flex flex-wrap items-end gap-4">
		<p
			class="bg-background-2 drop-shadow-box max-w-8/12 drop-shadow-background-3 break-all rounded px-4 py-1"
		>
			{data.meta.file_name}
		</p>
		<p class="bg-background-2 drop-shadow-box drop-shadow-background-3 h-min rounded px-4 py-1">
			{prettyBytes(data.meta.size)}
		</p>
		<p class="bg-background-2 drop-shadow-box drop-shadow-background-3 h-min rounded px-4 py-1">
			{fuckery_rust_time_to_date(data.meta.created_at).toLocaleString()}
		</p>
		<a href={data.url} class="text-text-2 underline">View raw</a>
	</div>

	<div
		class="bg-background-2 drop-shadow-box drop-shadow-background-3 max-h-9/12 flex items-center justify-center rounded p-2"
	>
		<!-- fix some more robust system on how to handle the incoming file -->
		{#if data.meta.mime_type.startsWith('video') && !(browser && navigator?.userAgent?.includes('Firefox') && data.meta.mime_type == 'video/x-matroska')}
			<!-- svelte-ignore a11y_media_has_caption -->
			<video class="max-h-full w-full" src={data.url} controls> </video>
		{:else if data.meta.mime_type.startsWith('image')}
			<img class="max-h-full w-full object-contain" src={data.url} alt="" />
		{:else if data.meta.mime_type.startsWith('audio')}
			<audio class="max-h-full w-full" src={data.url} controls></audio>
		{:else if data.meta.mime_type.startsWith('text')}
			{#await fetch(data.url)}
				<p>Loading text...</p>
			{:then response}
				{#await response.text()}
					<p>Loading text content...</p>
				{:then text}
					<div class="max-h-full w-full overflow-y-auto">
						<pre class="text-wrap break-all">{text}</pre>
					</div>
				{:catch error}
					<p>Error loading text content: {error.message}</p>
				{/await}
			{:catch error}
				<p>Error loading text: {error.message}</p>
			{/await}
		{:else}
			<div class="flex flex-col items-center justify-center gap-1">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-24"
					><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path
						d="M14 2v4a2 2 0 0 0 2 2h4"
					/></svg
				>

				<p class="text-wrap break-all text-center text-xl">
					{data.meta.file_name}
				</p>
				<p class="text-text-2 text-sm">Can't preview this file type</p>
			</div>
		{/if}
	</div>

	<div
		class="bg-background-2 drop-shadow-box drop-shadow-background-3 mb-5 flex flex-wrap justify-start gap-3 rounded px-4 py-2"
	>
		<!-- always for everyone-->
		<div class="flex flex-wrap gap-1">
			<button
				onclick={async () => {
					await navigator.clipboard.writeText(get_preview_link(data.id));

					// force enable public access
					if (data.has_token && data.meta.access == 0) {
						await change_access_with_id(data.meta.id, 1);
						await invalidateAll();
					}

					notification.success('Copied preview link to clipboard');
				}}
				aria-label="Copy preview link"
				title="Copy preview link"
				class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-7"
					><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path
						d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
					/></svg
				>
			</button>

			<button
				onclick={async () => {
					await navigator.clipboard.writeText(data.raw_url);

					// force enable public access
					if (data.has_token && data.meta.access == 0) {
						await change_access_with_id(data.meta.id, 1);
						await invalidateAll();
					}

					notification.success('Copied download link to clipboard');
				}}
				aria-label="Copy download link"
				title="Copy download link"
				class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-7"
					><path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path
						d="M14 2v4a2 2 0 0 0 2 2h4"
					/></svg
				>
			</button>

			<button
				onclick={() => {
					qr_dialog_open = true;
					qr_file_id = data.id;
				}}
				aria-label="Show QR Code"
				title="Show QR Code"
				class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-7"
					><rect width="5" height="5" x="3" y="3" rx="1" /><rect
						width="5"
						height="5"
						x="16"
						y="3"
						rx="1"
					/><rect width="5" height="5" x="3" y="16" rx="1" /><path
						d="M21 16h-3a2 2 0 0 0-2 2v3"
					/><path d="M21 21v.01" /><path d="M12 7v3a2 2 0 0 1-2 2H7" /><path d="M3 12h.01" /><path
						d="M12 3h.01"
					/><path d="M12 16v.01" /><path d="M16 12h1" /><path d="M21 12v.01" /><path
						d="M12 21v-1"
					/></svg
				>
			</button>
			<button
				onclick={() => {
					const a = document.createElement('a');
					a.href = data.raw_url;
					document.body.appendChild(a);
					a.click();
					document.body.removeChild(a);
				}}
				aria-label="Download"
				title="Download"
				class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-7"
					><path d="M12 15V3" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path
						d="m7 10 5 5 5-5"
					/></svg
				>
			</button>
		</div>

		{#if data.has_token}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="text-text-2/40 w-6"
				><path d="M12 3v18" /><path d="m16 16 4-4-4-4" /><path d="m8 8-4 4 4 4" /></svg
			>

			<div class="flex flex-wrap gap-1">
				<button
					onclick={async () => {
						history.back();
						await invalidateAll();
					}}
					aria-label="Go back"
					title="Go back"
					class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="w-7"><path d="m12 19-7-7 7-7" /><path d="M19 12H5" /></svg
					>
				</button>
				<button
					onclick={async () => {
						await change_access_with_id(data.meta.id, data.meta.access == 0 ? 1 : 0);
						await invalidateAll();
					}}
					aria-label="Change access"
					title="Change access"
					class="bg-background-1 text-text-2 hover:text-text h-full cursor-pointer rounded p-1 transition-colors"
				>
					{#if data.meta.access == 0}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="w-7"
							><path
								d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49"
							/><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" /><path
								d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143"
							/><path d="m2 2 20 20" /></svg
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
							class="w-7"
							><path
								d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"
							/><circle cx="12" cy="12" r="3" /></svg
						>
					{/if}
				</button>
			</div>
		{/if}
	</div>
</div>

<QrCode bind:open={qr_dialog_open} bind:file_id={qr_file_id} bind:access={data.meta.access} />
