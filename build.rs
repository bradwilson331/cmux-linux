use std::env;
use std::path::PathBuf;

fn main() {
    // Static link pre-built libghostty.a (built by scripts/setup-linux.sh)
    println!("cargo:rustc-link-search=ghostty/zig-out/lib");
    println!("cargo:rustc-link-lib=static=ghostty");
    // libghostty.a requires OpenGL at link time
    println!("cargo:rustc-link-lib=dylib=GL");

    // Use pkg-config for GTK4/GLib system libraries that libghostty.a needs
    // at link time if they are not fully bundled in the static archive.
    // This is a soft best-effort; link errors reveal which ones are needed.
    if std::process::Command::new("pkg-config")
        .args(["--exists", "gtk4"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let gtk4_libs = std::process::Command::new("pkg-config")
            .args(["--libs", "gtk4"])
            .output()
            .expect("pkg-config gtk4 failed");
        let flags = String::from_utf8_lossy(&gtk4_libs.stdout);
        for flag in flags.split_whitespace() {
            if let Some(lib) = flag.strip_prefix("-l") {
                println!("cargo:rustc-link-lib=dylib={lib}");
            }
        }
    }

    // Re-run bindgen when ghostty.h changes (Plan 02 already patched it)
    println!("cargo:rerun-if-changed=ghostty.h");

    let bindings = bindgen::Builder::default()
        .header("ghostty.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Needed for types that reference C integer types
        .allowlist_item("ghostty_.*")
        .allowlist_item("GHOSTTY_.*")
        .generate()
        .expect("Unable to generate ghostty bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ghostty_sys.rs"))
        .expect("Couldn't write ghostty_sys.rs");
}
