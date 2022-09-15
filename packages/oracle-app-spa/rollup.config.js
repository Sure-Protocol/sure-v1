import svelte from 'rollup-plugin-svelte';
import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import livereload from 'rollup-plugin-livereload';
import { terser } from 'rollup-plugin-terser';
import alias from '@rollup/plugin-alias';
import sveltePreprocess from 'svelte-preprocess';
import copy from 'rollup-plugin-copy';
import scss from 'rollup-plugin-scss';
import replace from '@rollup/plugin-replace';
import css from 'rollup-plugin-css-only';
import url from '@rollup/plugin-url';
import json from '@rollup/plugin-json';
import typescript from '@rollup/plugin-typescript';
import builtins from 'rollup-plugin-node-builtins';
import nodePolyfills from 'rollup-plugin-polyfill-node';
import { babel } from '@rollup/plugin-babel';

import globals from 'rollup-plugin-node-globals';

const production = !process.env.ROLLUP_WATCH;

function serve() {
	let server;

	function toExit() {
		if (server) server.kill(0);
	}

	return {
		writeBundle() {
			if (server) return;
			server = require('child_process').spawn(
				'npm',
				['run', 'start', '--', '--dev'],
				{
					stdio: ['ignore', 'inherit', 'inherit'],
					shell: true,
				}
			);

			process.on('SIGTERM', toExit);
			process.on('exit', toExit);
		},
	};
}

export default {
	input: 'src/main.ts',
	output: {
		sourcemap: true,
		format: 'iife',
		file: 'public/build/main.js',
		inlineDynamicImports: true,
		name: 'oracle',
	},
	plugins: [
		svelte({
			preprocess: sveltePreprocess({ sourceMap: !production }),
			compilerOptions: {
				// enable run-time checks when not in production
				dev: !production,
			},
		}),
		babel({ babelHelpers: 'bundled' }),
		// we'll extract any component CSS out into
		// a separate file - better for performance
		copy({
			targets: [
				{ src: 'src/assets/*.ttf', dest: 'public/assets/' },
				{
					src: 'src/assets/icons/*.svg',
					dest: 'public/assets/icons/',
				},
			],
		}),
		css({ output: 'bundle.css' }),
		url(),
		json(),
		//globals(),
		builtins(),

		// If you have external dependencies installed from
		// npm, you'll most likely need these plugins. In
		// some cases you'll need additional configuration -
		// consult the documentation for details:
		// https://github.com/rollup/plugins/tree/master/packages/commonjs
		resolve({
			browser: true,
			extensions: ['.js', '.ts'],
			dedupe: ['svelte'],
			preferBuiltins: false,
		}),
		commonjs({
			transformMixedEsModules: true,
		}),
		typescript({
			sourceMap: !production,
			inlineSources: !production,
		}),
		nodePolyfills({
			include: ['buffer', 'Stream'],
		}),
		replace({
			'process.env.NODE_ENV': JSON.stringify('production'),
			'process.env.NODE_DEBUG': JSON.stringify(false),
			'process.env.PROGRAM_ID': JSON.stringify(
				'D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM'
			),
		}),
		// In dev mode, call `npm run start` once
		// the bundle has been generated
		!production && serve(),

		// Watch the `public` directory and refresh the
		// browser on changes when not in production
		!production && livereload('public'),

		// If we're building for production (npm run build
		// instead of npm run dev), minify
		production && terser(),

		alias({
			entries: [
				{ find: '$static', replacement: 'static' },
				{ find: '$lib', replacement: 'src/lib' },
				{ find: '$stores', replacement: 'src/store' },
				{ find: '$assets', replacement: 'src/assets' },
			],
		}),
	],
	watch: {
		clearScreen: false,
	},
};
