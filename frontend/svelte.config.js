import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		// Use adapter-node for CapRover deployment
		adapter: adapter(),

		// Path aliases for clean imports
		alias: {
			'@/*': './src/*',
			$components: './src/lib/components',
			$stores: './src/lib/stores',
			$types: './src/lib/types',
			$api: './src/lib/api',
			$hooks: './src/lib/hooks'
		}
	}
};

export default config;
