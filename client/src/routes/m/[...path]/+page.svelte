<script lang="ts">
	import FileHandler from '$lib/FileHandler.svelte';
	import type { PageProps } from './$types';
	import TopStatusBar from '$lib/TopStatusBar.svelte';
	import FileList from '$lib/FileList.svelte';

	const { data }: PageProps = $props();
</script>

<svelte:head>
	<title>Simply... Files</title>
</svelte:head>

<TopStatusBar file_system={data.file_system} storage_limit={data.storage_limit} />

<FileList files={data.files} path={data.path} />
