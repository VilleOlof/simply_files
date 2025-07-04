<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { copy_link, create_link, type FileSystemInfo, type StorageLimit } from '$lib';
	import prettyBytes from 'pretty-bytes';

	const {
		file_system,
		storage_limit
	}: {
		file_system: FileSystemInfo;
		storage_limit: StorageLimit;
	} = $props();
	let storage_percentage = $state((storage_limit.used / storage_limit.max) * 100);

	function upload_button() {
		const input = document.createElement('input');
		input.type = 'file';
		input.multiple = false;
		input.accept = '*';
		input.style.display = 'none';

		input.onchange = (e) => {
			// this moves the job to the DropFile component
			const files = (e.target as HTMLInputElement).files;
			dispatchEvent(new CustomEvent('manual-upload', { detail: { files } }));
		};

		document.body.appendChild(input);
		input.click();
		document.body.removeChild(input);
	}
</script>

<div
	class="bg-background-2 drop-shadow-box drop-shadow-background-3 mb-5 flex w-2/3 justify-between gap-3 px-8 py-2 xl:w-1/3"
>
	<div class="flex items-center gap-4">
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
					{prettyBytes(storage_limit.used)} / {prettyBytes(storage_limit.max)}
				</span>

				<!-- outside green bar -->
				<span
					class="absolute inset-0 flex items-center justify-center text-white"
					style="clip-path: inset(0 0 0 {storage_percentage}%);"
				>
					{prettyBytes(storage_limit.used)} / {prettyBytes(storage_limit.max)}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2" title={file_system.about}>
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
			<span class="text-text-1 hover:underline">{file_system.which}</span>
		</div>
	</div>

	<div class="flex items-center gap-4">
		<div class="flex">
			<button
				onclick={async () => {
					let link = await create_link();
					copy_link(link);
					await invalidateAll();
				}}
				class="bg-background-1 hover:bg-background-3 flex cursor-pointer items-center gap-2 rounded-l px-2 transition-colors"
				>+</button
			>

			<button
				onclick={() => goto('/m/.public_uploads')}
				class="bg-background-1 hover:bg-background-3 flex cursor-pointer items-center gap-2 rounded-r px-2 transition-colors"
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
						class="text-text-2 w-5"
						><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path
							d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
						/></svg
					>
				</span>
				Links</button
			>
		</div>

		<button
			onclick={upload_button}
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
</div>
