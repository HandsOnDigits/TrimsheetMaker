fn main() {
    println!("cargo:rerun-if-changed=icon.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("icon.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("Failed to embed Windows application icon");
    }
}
