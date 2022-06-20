import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import replace from '@rollup/plugin-replace';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import { terser } from 'rollup-plugin-terser';
import dotenv from 'dotenv';
dotenv.config();

const env = process.env.NODE_ENV;
export default {
	input: 'src/index.ts',
	plugins: [
		json(),
		commonjs(),
		nodeResolve({
			browser: true,
			extensions: ['.js', '.ts'],
			dedupe: ['bn.js', 'buffer'],
			preferBuiltins: false,
		}),
		typescript({
			tsconfig: './tsconfig.base.json',
			moduleResolution: 'node',
			outDir: 'types',
			target: 'es2020',
			outputToFilesystem: false,
		}),
		replace({
			preventAssignment: true,
			values: {
				'process.env.NODE_ENV': JSON.stringify('production'),
				'process.env.ANCHOR_BROWSER': JSON.stringify(true),
			},
		}),
		terser(),
	],
	output: {
		file: 'dist/browser/index.js',
		format: 'esm',
		sourcemap: true,
	},
};
