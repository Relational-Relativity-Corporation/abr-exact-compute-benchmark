//! Records the compiler and build settings in the benchmark binary (provenance).
fn main() {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let vv = std::process::Command::new(rustc).arg("-vV").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().replace('\n', "; ")).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=BUILD_RUSTC_VV={vv}");
    for k in ["PROFILE", "OPT_LEVEL", "DEBUG", "TARGET"] {
        println!("cargo:rustc-env=BUILD_{k}={}", std::env::var(k).unwrap_or_default());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
