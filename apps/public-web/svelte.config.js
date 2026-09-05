import adapter from '@sveltejs/adapter-node';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://svelte.dev/docs/kit/adapters for more information about adapters.
		adapter: adapter(),
		csp: {
			directives: {
				'connect-src': [
					'self',
					'http://localhost:8081',
					'http://localhost:8000',
					'https://slv-gateway.wign.dev',
					'https://api-engine.wign.dev',
					'ws://localhost:8090',
					'ws://localhost:8020',
					'https:',
					'wss://slv-realtime.wign.dev',
					'wss://api-atlsd.wign.cloud',
					'wss://realtime-engine.wign.dev',
					'http://api-gateway:8000'
				]
			}
		}
	}
};

export default config;
