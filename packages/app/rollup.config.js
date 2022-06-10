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

export default {
	input: 'src/index.tsx',
	output: {
		file: 'dist_up/bundle.js',
		format: 'iife',
		sourcemap: true,
	},
	plugins: [
		svgr(),
		scss(),
		typescript(),
		json(),
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
