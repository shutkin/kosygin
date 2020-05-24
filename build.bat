cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir . target\wasm32-unknown-unknown\release\kosygin.wasm
rem wasm-gc kosygin_bg.wasm
copy /Y kosygin_bg.wasm c:\nginx\html\kosygin_bg.wasm
