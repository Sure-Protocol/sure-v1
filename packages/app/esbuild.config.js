const path = require('path');
const { sassPlugin } = require('esbuild-sass-plugin');
const { build } = require('esbuild');
const svgrPlugin = require('esbuild-plugin-svgr');
const alias = require('esbuild-plugin-alias');
const { copy } = require('esbuild-plugin-copy');
build({
	entryPoints: ['src/index.tsx'],
	bundle: true,
	minify: true,
	sourcemap: true,
	outdir: 'dist_es',
	loader: {
		'.html': 'text',
		'.ttf': 'file',
	},
	define: {
		'process.env.BROWSER': 'true',
		'process.env.NODE_ENV': 'devnet',
	},
	plugins: [
		sassPlugin(),
		svgrPlugin({ ref: true }),
		alias({
			stream: path.resolve(
				__dirname,
				'./../../node_modules/stream-browserify/index.js'
			),
			crypto: path.resolve(
				__dirname,
				'./../../node_modules/crypto-browserify/index.js'
			),
		}),
	],
});
