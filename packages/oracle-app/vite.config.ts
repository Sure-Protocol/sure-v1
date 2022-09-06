import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';
import path from 'path';
import inject from '@rollup/plugin-inject';
import nodePolyfills from 'rollup-plugin-node-polyfills';

const config: UserConfig = {
	plugins: [sveltekit()],
	logLevel: 'warn',
	optimizeDeps: {
		include: ['@solana/web3.js', 'buffer'],
		esbuildOptions: {
			target: 'esnext',
			plugins: [NodeGlobalsPolyfillPlugin({ buffer: true })]
		}
	},
	resolve: {
		alias: {
			$assets: path.resolve('src/assets'),
			stream: 'rollup-plugin-node-polyfills/polyfills/stream'
		}
	},
	define: {
		'process.env.BROWSER': true,
		'process.env.NODE_DEBUG': JSON.stringify(''),
		'process.env.SURE_ENV': JSON.stringify(process.env.SURE_ENV)
	},
	build: {
		target: 'esnext',
		commonjsOptions: {
			transformMixedEsModules: true
		},
		rollupOptions: {
			plugins: [inject({ Buffer: ['buffer', 'Buffer'] }), nodePolyfills({ crypto: true })]
		}
	},
	kit: {
		alias: {
			'@solana/spl-token': './node_modules/@solana/spl-token',
			'$assets/*': 'src/assets/*',
			'$lib/*': 'src/lib/*'
		}
	}
};

export default config;
