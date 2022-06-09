import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [
		react({
			babel: {
				babelrc: true,
			},
		}),
	],
	build: {
		target: ['es6'],
		outDir: './public',
	},
	define: {
		'process.env': {},
		global: {},
	},
	optimizeDeps: {
		include: ['buffer'],
	},
});
