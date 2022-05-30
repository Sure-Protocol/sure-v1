import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import replace from '@rollup/plugin-replace';
import commonjs from '@rollup/plugin-commonjs';

import json from '@rollup/plugin-json';
import { terser } from 'rollup-plugin-terser';

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
				'process.env.NODE_ENV': JSON.stringify(env),
				'process.env.ANCHOR_BROWSER': JSON.stringify(true),
			},
		}),
		terser(),
	],
	// external: [
	// 	'@project-serum/anchor',
	// 	'@solana/spl-token',
	// 	'@metaplex/js',
	// 	'@solana/web3.js',
	// ],
	output: {
		file: 'dist/browser/index.js',
		format: 'esm',
		sourcemap: true,
	},
};
