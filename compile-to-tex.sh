cargo build --manifest-path chalmers-thesis/Cargo.toml --release --target wasm32-wasi
cp chalmers-thesis/target/wasm32-wasi/release/chalmers-thesis.wasm ./
./modmark --assets "assets" --allow-every-module main.mdm out/thesis.tex