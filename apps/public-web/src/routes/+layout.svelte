<script lang="ts">
	import './layout.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>SLV — Real-time Market Intelligence</title>
	<meta
		name="description"
		content="Friendly real-time market analytics for forex, crypto, equities, news, calendars, and alerts."
	/>
	<meta name="theme-color" content="#11110F" />
</svelte:head>

{@render children()}
