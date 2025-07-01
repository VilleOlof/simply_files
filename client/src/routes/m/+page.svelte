<script>
	import { PUBLIC_BACKEND } from '$env/static/public';
	import DropFile from '$lib/DropFile.svelte';
</script>

<h1>Home protected</h1>

<button
	onclick={async () => {
		let res = await fetch(`${PUBLIC_BACKEND}/m/check`, { credentials: 'include' });
	}}>check</button
>

<DropFile endpoint={'/m/upload'} />
