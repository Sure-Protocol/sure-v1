import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import sveltePreprocess from 'svelte-preprocess';
import inject from '@rollup/plugin-inject';
import resolve from '@rollup/plugin-node-resolve';
import * as path from 'path';

const isProd = process.env.NODE_ENV === 'production';
/** @type {import('vite').UserConfig} */
export default defineConfig({
	build: {
		outDir: 'dist',
		target: 'esnext',
		rollupOptions: {
			plugins: [inject({ Buffer: ['Buffer', 'Buffer'] })],
		},
	},
	plugins: [
		svelte({
			preprocess: sveltePreprocess({ sourceMap: !isProd }),
			compilerOptions: {
				// enable run-time checks when not in production
				dev: !isProd,
			},
		}),
		resolve({
			extensions: ['.ts', '.js'],
		}),
	],
	optimizeDeps: {
		esbuildOptions: {
			target: 'esnext',
		},
	},
	resolve: {
		extensions: ['.ts', '.js'],
		alias: [
			{ find: '$static', replacement: path.resolve(__dirname, './static') },
			{ find: '$lib', replacement: path.resolve(__dirname, './src/lib') },
			{ find: '$stores', replacement: path.resolve(__dirname, './src/store') },
			{ find: '$assets', replacement: path.resolve(__dirname, './src/assets') },
		],
	},
	define: {
		'process.env.NODE_ENV': JSON.stringify('production'),
		'process.env.NODE_DEBUG': JSON.stringify(false),
		'process.env.PROGRAM_ID': JSON.stringify(
			'D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM'
		),
	},
});
