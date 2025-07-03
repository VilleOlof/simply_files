<script lang="ts">
	import { goto } from '$app/navigation';
	import { GITHUB_URL, login, logout } from '$lib';
	import type { PageProps } from './$types';

	let token = $state('');

	const { data }: PageProps = $props();
</script>

<p class="text-2xl">File sharing done easily</p>

<div class="mt-4"></div>

<p class="w-1/4 text-balance text-center text-xl">
	<a href={GITHUB_URL} class="text-primary drop-shadow-secondary/50 drop-shadow hover:underline"
		>Simply... Files</a
	>
	makes it easy to upload files, organize them and share them via one click.<br />
</p>

<div class="mt-4"></div>

<p class="text-2xl">Features</p>
<div>
	<p>- One-time links for others to upload files</p>
	<p>- Easy to use interface</p>
	<p>- Secure & fast, backend in Rust</p>
	<p>- Easy to self-host, prebuilt binaries</p>
	<!-- <p>- Folders & tags to help you organize</p> -->
	<p>- Folders to help you organize</p>
	<p>- No ads, no payment, 100% free</p>
	<p>- Store your files locally or via SFTP</p>
</div>

<div class="mt-12"></div>

<p class="text-center">
	This is an already hosted instance. <br />To host your own, check out the
	<a href={GITHUB_URL} class="underline hover:opacity-80">Github</a>
</p>

<div class="mt-12"></div>

<div class="flex flex-col items-center gap-4">
	{#if data.has_token}
		<p class="text-xl">You're already logged in!</p>
		<div class="flex gap-4">
			<button
				onclick={async () => {
					await goto('/m');
				}}
				class="bg-background-2 drop-shadow-background-3 drop-shadow-box hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-4 py-1 text-xl transition-all"
			>
				View panel</button
			>
			<button
				onclick={async () => {
					if (await logout()) window.location.reload();
				}}
				class="bg-background-2 drop-shadow-background-3 drop-shadow-box hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-4 py-1 text-xl transition-all"
			>
				Logout</button
			>
		</div>
	{:else}
		<p class="text-xl">Own this instance? Want to log in?</p>
		<div class="flex flex-col gap-4">
			<input
				bind:value={token}
				type="password"
				placeholder="token..."
				class="bg-background-2 drop-shadow-background-3 drop-shadow-box rounded px-4 py-1 text-xl outline-none"
			/>
			<button
				onclick={async () => {
					await login(token);
				}}
				class="bg-background-2 drop-shadow-background-3 drop-shadow-box hover:bg-secondary active:bg-primary text-shadow-lg text-shadow-background-1/50 cursor-pointer rounded px-4 py-1 text-xl transition-all"
			>
				Login</button
			>
		</div>
	{/if}
</div>
