<script lang="ts">
	import DropFile from '$lib/DropFile.svelte';
	import type { PageProps } from './$types';
	import TopStatusBar from '$lib/TopStatusBar.svelte';
	import FileList from '$lib/FileList.svelte';

	const { data }: PageProps = $props();
</script>

<TopStatusBar file_system={data.file_system} storage_limit={data.storage_limit} />

<FileList files={data.files} path={data.path} />

<DropFile endpoint={'/m/upload'} />
