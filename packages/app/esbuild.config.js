const path = require('path');
const { sassPlugin } = require('esbuild-sass-plugin');
const { build } = require('esbuild');
const svgrPlugin = require('esbuild-plugin-svgr');
const alias = require('esbuild-plugin-alias');

build({
	entryPoints: ['src/index.tsx'],
	bundle: true,
	minify: true,
	sourcemap: true,
	outfile: 'dist_es/index.js',
	loader: {
		'.html': 'text',
		'.ttf': 'file',
	},
	plugins: [
		sassPlugin(),
		svgrPlugin(),
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
