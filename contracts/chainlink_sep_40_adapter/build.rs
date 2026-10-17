// Every cross-contract call pays for the callee's whole linear memory: the 1 MB default stack cost
// ~1 MB per instantiation, and 64 KB took the gated Liqvid batch from 101 % to 66 % of pubnet memory.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_FAMILY").is_ok_and(|family| family == "wasm") {
        println!("cargo:rustc-link-arg-cdylib=-zstack-size=65536");
    }
}
