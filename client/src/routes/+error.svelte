<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="mt-4 flex flex-col items-center sm:flex-row sm:gap-8">
	<h1 class="text-primary">{page.status}</h1>
	<div class="flex flex-col items-center gap-4 text-xl sm:items-start">
		<p class="text-center">Something went wrong</p>
		<p class="text-text-2 text-center">{page.error?.message}</p>
	</div>
</div>

<img
	src="/error.webp"
	alt=""
	class="drop-shadow-box drop-shadow-background-3 h-9/12 my-8 object-contain md:w-2/5"
/>
