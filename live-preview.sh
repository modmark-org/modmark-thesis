modmark="cargo run --release --manifest-path ${MODMARK_PATH}/cli/Cargo.toml --"

cargo build --manifest-path chalmers-thesis/Cargo.toml --release --target wasm32-wasi
cp chalmers-thesis/target/wasm32-wasi/release/chalmers-thesis.wasm ./
$modmark compile --assets "." -A main.mdm