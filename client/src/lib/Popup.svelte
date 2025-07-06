<script lang="ts">
	import { onMount, type Snippet } from 'svelte';

	let { open = $bindable(), children }: { open: boolean; children: Snippet } = $props();

	onMount(() => {
		const handleKeydown = (event: KeyboardEvent) => {
			if (!open) return;

			if (event.key === 'Escape') {
				open = false;
			}
		};

		window.addEventListener('keydown', handleKeydown);

		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

{#if open}
	<div
		class="absolute left-0 top-0 z-50 flex h-full w-full items-center justify-center bg-black/70"
	>
		<div class="bg-background-3 w-11/12 rounded p-4 md:w-auto">
			{@render children()}
		</div>
	</div>
{/if}
