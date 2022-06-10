module.exports = {
	plugins: [],
	webpack: {
		configure: {
			resolve: {
				fallback: {
					stream: require.resolve('stream-browserify'),
					crypto: require.resolve('crypto-browserify'),
				},
			},
		},
	},
};
