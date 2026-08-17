use std::env;
use std::path::PathBuf;

fn main() {
    // Get the absolute path to the project directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ghostty_lib_path = format!("{}/ghostty/zig-out/lib", manifest_dir);

    // Static link pre-built libghostty.a (built by scripts/setup-linux.sh)
    // Use absolute path to ensure it's found
    println!("cargo:rustc-link-search=native={}", ghostty_lib_path);
    println!("cargo:rustc-link-lib=static=ghostty");

    // libghostty.a vendors glslang, SPIRV-Cross and Dear ImGui, which Zig
    // compiles against its own bundled libc++ (symbols in the `std::__1::` ABI
    // namespace), not the system libstdc++. A static archive carries no C++
    // runtime, so we link the matching libc++/libc++abi that setup-linux.sh
    // copied into ghostty/zig-out/lib. Order matters: both must follow
    // -lghostty, and libc++abi must follow libc++.
    println!("cargo:rustc-link-lib=static=c++");
    println!("cargo:rustc-link-lib=static=c++abi");

    // Same story for libxml2, which the vendored fontconfig (fcxml.o) needs.
    // We use the Zig-built copy rather than the system one so it matches the
    // headers those fontconfig objects were compiled against.
    println!("cargo:rustc-link-lib=static=xml2");

    // NOTE: simdutf.o and libhighway used to be dug out of ghostty/.zig-cache
    // and linked explicitly, because the old pinned libghostty.a referenced
    // them without providing them. The current ghostty archives both
    // (`simdutf.o` and `libhighway_zcu.o` are members of libghostty.a), so
    // linking them again produces duplicate symbol errors. Verify with
    // `ar t ghostty/zig-out/lib/libghostty.a | grep -E 'simdutf|highway'`
    // before reintroducing either.

    // NOTE: stubs.o used to be linked here to satisfy glslang / SPIRV-Cross /
    // ImGui symbols that libghostty.a referenced but did not provide. Ghostty
    // now vendors real implementations of all three, so linking the stubs
    // produces ~87 duplicate symbol errors. Do not reintroduce them; if a new
    // undefined symbol appears, check whether ghostty vendors it first.

    // Link the GLAD loader object file — provides gladLoaderLoadGLContext and
    // gladLoaderUnloadGLContext which are needed by ghostty's OpenGL renderer.
    println!("cargo:rustc-link-arg={}/glad.o", manifest_dir);

    // Rebuild if the glad source changes
    println!("cargo:rerun-if-changed=ghostty/vendor/glad/src/gl.c");
    println!("cargo:rerun-if-changed=ghostty/vendor/glad/include/glad/gl.h");

    // libghostty.a requires these system libraries at link time
    println!("cargo:rustc-link-lib=dylib=GL");
    // NOTE: no -lstdc++ here. Ghostty's C++ is compiled against libc++ (linked
    // statically above); pulling in the system libstdc++ as well would mix two
    // C++ runtimes in one binary.
    println!("cargo:rustc-link-lib=dylib=gcc_s"); // unwinder
    println!("cargo:rustc-link-lib=dylib=fontconfig");
    println!("cargo:rustc-link-lib=dylib=freetype");

    // Try to link the versioned onig library if dev package isn't installed
    if std::process::Command::new("pkg-config")
        .args(["--exists", "oniguruma"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        println!("cargo:rustc-link-lib=dylib=onig");
    } else if std::path::Path::new("/usr/lib/x86_64-linux-gnu/libonig.so.5").exists() {
        // Link to the versioned library file directly
        println!("cargo:rustc-link-arg=/usr/lib/x86_64-linux-gnu/libonig.so.5");
    }

    // glslang is optional - ghostty can work without it
    // We'll skip it for now since it's not installed

    // Use pkg-config for GTK4/GLib system libraries that libghostty.a needs
    // at link time if they are not fully bundled in the static archive.
    // This is a soft best-effort; link errors reveal which ones are needed.
    if std::process::Command::new("pkg-config")
        .args(["--exists", "gtk4"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        // Emit link-search dirs from the .pc file location (handles extracted dev packages).
        // pkg-config --variable=pcfiledir emits the directory containing the .pc file; the
        // sibling directory (../lib or the pkgconfig parent) contains the .so linker stubs.
        for pkg in &["gtk4", "graphene-gobject-1.0"] {
            let pcdir_out = std::process::Command::new("pkg-config")
                .args(["--variable=pcfiledir", pkg])
                .output();
            if let Ok(out) = pcdir_out {
                let pcdir = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !pcdir.is_empty() {
                    // pkgconfig dir is typically .../lib/x86_64-linux-gnu/pkgconfig;
                    // the parent contains the .so symlinks.
                    let libdir = std::path::Path::new(&pcdir)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if !libdir.is_empty() {
                        println!("cargo:rustc-link-search=native={libdir}");
                    }
                }
            }
        }

        let gtk4_libs = std::process::Command::new("pkg-config")
            .args(["--libs", "gtk4"])
            .output()
            .expect("pkg-config gtk4 failed");
        let flags = String::from_utf8_lossy(&gtk4_libs.stdout);
        for flag in flags.split_whitespace() {
            if let Some(lib) = flag.strip_prefix("-l") {
                println!("cargo:rustc-link-lib=dylib={lib}");
            } else if let Some(path) = flag.strip_prefix("-L") {
                println!("cargo:rustc-link-search=native={path}");
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
