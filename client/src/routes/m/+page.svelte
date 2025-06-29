<script>
	import { PUBLIC_BACKEND } from '$env/static/public';
</script>

<h1>Home protected</h1>

<button
	onclick={async () => {
		let res = await fetch(`${PUBLIC_BACKEND}/m/check`, { credentials: 'include' });
	}}>check</button
>
