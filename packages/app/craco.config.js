const path = require('path');
module.exports = {
	plugins: [],
	webpack: {
		configure: {
			resolve: {
				alias: {
					stream: path.resolve(
						__dirname,
						'./../../node_modules/stream-browserify/index.js'
					),
					crypto: path.resolve(
						__dirname,
						'./../../node_modules/crypto-browserify/index.js'
					),
				},
			},
		},
	},
};
