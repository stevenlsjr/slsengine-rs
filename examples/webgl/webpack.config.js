const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require("path");

const crateRoot = path.resolve(__dirname, "../..");

module.exports = {
  entry: "./src/example-webgl",
  devtool: "source-maps",
  mode: "development",

  plugins: [
    new HtmlWebpackPlugin(),
    new WasmPackPlugin({
      crateDirectory: path.resolve(crateRoot, "slsengine-backend-webgl")
    })
  ]
};
