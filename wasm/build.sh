wasm-pack build --target nodejs --out-dir pkg-node
wasm-pack build --target bundler --out-dir pkg-bundler

rm pkg-node/package.json
rm pkg-node/README.md
rm pkg-bundler/package.json
rm pkg-bundler/README.md
