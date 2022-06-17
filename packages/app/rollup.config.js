import babel from '@rollup/plugin-babel';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import replace from '@rollup/plugin-replace';
import typescript from '@rollup/plugin-typescript';
import scss from 'rollup-plugin-scss';
import svgr from '@svgr/rollup';
import json from '@rollup/plugin-json';
import nodePolyfills from 'rollup-plugin-polyfill-node';
import copy from 'rollup-plugin-copy';
import url from '@rollup/plugin-url';
import inject from '@rollup/plugin-inject';

export default {
	input: 'src/index.tsx',
	output: {
		file: 'dist_up/index.js',
		format: 'esm',
	},
	external: ['websocket'],
	plugins: [
		json(),
		commonjs(),
		nodePolyfills({
			include: ['buffer', 'stream', 'crypto', 'serialize'],
		}),
		nodeResolve({
			browser: true,
			extensions: ['.js', '.ts'],
			dedupe: ['bn.js', 'buffer'],
			preferBuiltins: false,
		}),
		inject({ Buffer: ['buffer', 'Buffer'] }),
		url(),
		svgr(),
		scss(),
		typescript(),
		copy({
			targets: [
				{ src: 'src/assets/*.ttf', dest: 'dist_up/assets/' },
				{ src: 'src/index.html', dest: 'dist_up/' },
				{
					src: 'src/assets/icons/sureLogo.svg',
					dest: 'dist_up/assets/icons/',
				},
			],
		}),
		replace({
			'process.env.NODE_ENV': JSON.stringify('development'),
			'process.env.NETWORK': JSON.stringify('devnet'),
			'process.env.PROGRAM_ID': JSON.stringify(
				'D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM'
			),
		}),
	],
};
