const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require('html-webpack-plugin')
const dist = path.resolve(__dirname, "target/dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./web/index.js"
  },
  output: {
    path: dist,
    filename: "[name].[contenthash].js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "web/static")
    ]),

    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--dev"
    }),
    new HtmlWebpackPlugin()
  ]
};