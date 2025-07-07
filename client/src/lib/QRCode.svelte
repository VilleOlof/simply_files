<script lang="ts">
	import { PUBLIC_BACKEND } from '$env/static/public';
	import Popup from './Popup.svelte';

	let {
		open = $bindable(),
		link_id = $bindable(),
		file_id = $bindable(),
		access = $bindable()
	}: {
		open: boolean;
		link_id?: string | undefined;
		file_id?: string | undefined;
		access?: number | undefined;
	} = $props();

	let latest_qr_url: string | undefined = $state(undefined);
	async function get_qr_image(id: string): Promise<string> {
		const response = await fetch(
			`${PUBLIC_BACKEND}/qr/${link_id ? 'link' : 'file'}/${id}${file_id ? '?preview_link=true' : ''}`,
			{
				credentials: 'include'
			}
		);
		if (!response.ok) {
			throw new Error('Failed to fetch QR code image');
		}

		const url = URL.createObjectURL(await response.blob());
		if (latest_qr_url) {
			URL.revokeObjectURL(latest_qr_url);
		}
		latest_qr_url = url;

		return url;
	}

	function get_id(): string {
		if (link_id) {
			return link_id;
		} else if (file_id) {
			return file_id;
		}
		throw new Error('No valid ID provided for QR code generation');
	}

	let qr_promise: Promise<string> | undefined = $state(undefined);
	$effect(() => {
		if (open && (link_id || file_id)) {
			qr_promise = get_qr_image(get_id());
		} else {
			qr_promise = undefined;
			if (latest_qr_url) {
				URL.revokeObjectURL(latest_qr_url);
				latest_qr_url = undefined;
			}
		}
	});
</script>

<Popup bind:open>
	<div class="bg-background-3 flex flex-col gap-4 rounded p-4">
		{#if qr_promise}
			{#await qr_promise}
				<p class="text-balance text-center">Loading QR Code...</p>
			{:then qr_url}
				<img src={qr_url} alt="QR Code" class="drop-shadow-box drop-shadow-background-1" />
			{:catch _}
				<p class="text-red-500">Failed to load QR Code</p>
			{/await}
		{/if}

		{#if access !== undefined && access !== null && access == 0}
			<div class="flex w-full justify-center">
				<p class="max-w-80 text-balance text-center text-red-500/80">
					This file is private, you won't able to share it with anyone else
				</p>
			</div>
		{/if}

		<div class="flex gap-2">
			{#if latest_qr_url}
				<button
					onclick={() => {
						if (!latest_qr_url) return;
						let a = document.createElement('a');
						a.href = latest_qr_url;
						a.download = `qr_code_${get_id()}.png`;
						a.click();
					}}
					aria-label="Download QR Code"
					title="Download QR Code"
					class="bg-background-1 hover:bg-background-2 cursor-pointer rounded px-2 py-2 transition-colors"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="w-8"
						><path d="M12 15V3" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path
							d="m7 10 5 5 5-5"
						/></svg
					>
				</button>
			{/if}

			<button
				onclick={() => {
					open = false;
				}}
				class="bg-background-1 hover:bg-background-2 w-full cursor-pointer rounded px-4 py-2 transition-colors"
			>
				Close
			</button>
		</div>
	</div>
</Popup>
