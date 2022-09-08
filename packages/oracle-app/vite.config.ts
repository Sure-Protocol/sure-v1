import { sveltekit } from '@sveltejs/kit/vite';
import { readFileSync } from 'fs';
import { defineConfig } from 'vite';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';
import path from 'path';
import inject from '@rollup/plugin-inject';
import { esbuildCommonjs, viteCommonjs } from '@originjs/vite-plugin-commonjs';

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));

export default defineConfig(({ command }) => {
	return {
		plugins: [viteCommonjs(), sveltekit()],
		// logLevel: 'warn',
		optimizeDeps: {
			esbuildOptions: {}
		},
		ssr:
			command === 'build' &&
			{
				//external: ['@saberhq/anchor-contrib']
				//noExternal: true
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
			target: 'esnext',
			rollupOptions: {
				external: [
					'@saberhq/anchor-contrib',
					'@solana/web3.js',
					'@saberhq/solana-contrib',
					'@saberhq/token-utils',
					'@gokiprotocol/client',
					'bn.js',
					'@project-serum/anchor',
					'tiny-invariant',
					'tslib',
					'superstruct'
				]
			}
			// commonjsOptions: {
			// 	transformMixedEsModules: false
			// },
			// rollupOptions: {
			// 	plugins: [inject({ Buffer: ['buffer', 'Buffer'] }), nodePolyfills({ crypto: true })]
			// }
		}
	};
});
