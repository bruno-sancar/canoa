fn main() {
    if let Ok(duracao) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        println!("cargo:rustc-env=CANOA_BUILD_EPOCH={}", duracao.as_secs());
    }
    tauri_build::build()
}
