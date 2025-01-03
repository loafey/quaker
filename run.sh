cargo build -p default --target wasm32-unknown-unknown
echo "    Compiled default QWAK file to: \"target/wasm32-unknown-unknown/debug/default.wasm\"" 
cargo run -p qwak