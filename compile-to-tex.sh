cargo build --manifest-path chalmers-thesis/Cargo.toml --release --target wasm32-wasi
cp chalmers-thesis/target/wasm32-wasi/release/chalmers-thesis.wasm ./
./modmark --no-prompts main.mdm out/thesis.tex