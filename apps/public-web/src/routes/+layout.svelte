<script lang="ts">
	import './layout.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>Fio — Real-time Market Intelligence</title>
	<meta name="description" content="Real-time forex, crypto, and equity market intelligence. Live prices, news feeds, economic calendar, and Discord bot integration." />
	<meta name="theme-color" content="#09090B" />
</svelte:head>

{@render children()}
