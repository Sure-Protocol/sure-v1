import serve from 'rollup-plugin-serve';
import livereload from 'rollup-plugin-livereload';
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

export default {
	input: 'src/index.tsx',
	output: {
		file: 'dist_up/index.js',
		format: 'iife',
		sourcemap: true,
	},
	plugins: [
		nodePolyfills(),
		svgr(),
		scss(),
		typescript(),
		json(),
		copy({
			targets: [{ src: 'src/assets/*.ttf', dest: 'dist_up/' }],
		}),
		nodeResolve({
			extensions: ['.js', '.ts'],
		}),
		replace({
			'process.env.NODE_ENV': JSON.stringify('development'),
		}),
		babel({
			presets: ['@babel/preset-react'],
		}),
		commonjs(),
	],
};
