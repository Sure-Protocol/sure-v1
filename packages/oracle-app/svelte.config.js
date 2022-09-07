import node from '@sveltejs/adapter-node';
import preprocess from 'svelte-preprocess';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://github.com/sveltejs/svelte-preprocess
	// for more information about preprocessors
	preprocess: preprocess(),
	vitePlugin: {},
	kit: {
		adapter: node(),
		// Override http methods in the Todo forms
		methodOverride: {
			allowed: ['PATCH', 'DELETE']
		},
		alias: {
			'$static/*': 'static/*',
			'$lib/*': 'src/lib/*',
			'$stores/*': 'src/stores/*',
			'$assets/*': 'src/assets/*'
		}
	}
};

export default config;
