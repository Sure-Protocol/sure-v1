import { sveltekit } from '@sveltejs/kit/vite';
import { readFileSync } from 'fs';
import { defineConfig } from 'vite';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';
import path from 'path';
import inject from '@rollup/plugin-inject';
const pkg = JSON.parse(readFileSync('package.json', 'utf8'));

export default defineConfig(({ command }) => {
	return {
		plugins: [sveltekit()],
		// logLevel: 'warn',
		optimizeDeps: {
			esbuildOptions: {
				target: 'esnext'
			}
		},
		ssr: command === 'build' && {
			external: Object.keys(pkg.dependencies),
			noExternal: ['@tribecahq/tribeca-sdk', 'oracle-sdk', '@surec/oracle']
		},
		resolve: {
			alias: {
				stream: 'rollup-plugin-node-polyfills/polyfills/stream',
				http: 'rollup-plugin-node-polyfills/polyfills/http',
				https: 'rollup-plugin-node-polyfills/polyfills/http',
				zlib: 'rollup-plugin-node-polyfills/polyfills/zlib'
			}
		},
		define: {
			//global: {},
			'process.env.BROWSER': true,
			'process.env.NODE_DEBUG': JSON.stringify(''),
			'process.env.SURE_ENV': JSON.stringify(process.env.SURE_ENV)
		},
		build: {
			target: 'esnext'
			// commonjsOptions: {
			// 	transformMixedEsModules: false
			// },
			// rollupOptions: {
			// 	plugins: [inject({ Buffer: ['buffer', 'Buffer'] }), nodePolyfills({ crypto: true })]
			// }
		}
	};
});
