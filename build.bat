cargo build --manifest-path ./contracts/Cargo.toml --target wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\*.wasm res