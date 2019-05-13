
const webpack = require('webpack');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const HtmlWebpackPlugin = require('html-webpack-plugin');
module.exports = {
    entry: './js/index',
    output: {
        filename: '[name].[hash].bundle.js'
    },
    devtool: 'inline-source-map',
    plugins: [
        new WasmPackPlugin({
            crateDirectory: __dirname
        }),
        new HtmlWebpackPlugin()
    ],
    devtool: 'inline-source-map'
};