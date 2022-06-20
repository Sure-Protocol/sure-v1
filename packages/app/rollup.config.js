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
import serve from 'rollup-plugin-serve';

const env = process.env.NODE_ENV;

export default {
	input: 'src/index.tsx',
	output: {
		file: 'dist_up/index.js',
		format: 'umd',
	},
	external: ['websocket'],
	plugins: [
		...[
			json(),
			commonjs(),
			nodePolyfills({
				include: ['buffer', 'stream', 'crypto'],
			}),
			nodeResolve({
				browser: true,
				extensions: ['.js', '.ts'],
				dedupe: ['bn.js', 'buffer'],
				preferBuiltins: false,
			}),
			url(),
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
				'process.env.NODE_ENV': JSON.stringify('production'),
			}),
		],
		env === 'development'
			? serve({ verbose: true, contentBase: 'dist_up', port: 3030 })
			: [],
	],
};
