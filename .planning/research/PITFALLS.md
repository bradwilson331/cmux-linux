# Domain Pitfalls

**Domain:** Linux packaging & distribution for Rust/GTK4 app with Zig and Go subcomponents
**Researched:** 2026-03-28
**Confidence:** HIGH (verified from codebase analysis, official docs, multiple community sources)

> **Scope note:** This file covers pitfalls specific to the v1.1 Linux Packaging & Distribution milestone. The v1.0 development pitfalls (threading model, focus routing, surface lifecycle, etc.) are archived and remain valid but are not repeated here.

---

## Critical Pitfalls

Mistakes that produce broken packages, runtime failures on user machines, or CI pipeline rewrites.

### Pitfall 1: libghostty.a Links Against Build-Host glibc -- Packages Fail on Older Distros

**What goes wrong:** cmux-linux statically links `libghostty.a` (built by Zig 0.15.2) but dynamically links `libGL`, `libstdc++`, `libfontconfig`, `libfreetype`, `libonig`, `libgcc_s`, and GTK4 itself (all visible in `build.rs`). The Zig-compiled portion targets a specific glibc baseline (Zig 0.15.x defaults to glibc 2.28), but the Rust binary + GTK4 bindings link against whatever glibc the build host provides. If CI builds on Ubuntu 24.04 (glibc 2.39), the final binary requires glibc 2.39 -- crashing on Ubuntu 22.04 (glibc 2.35) with "version GLIBC_2.XX not found."

**Why it happens:** The split toolchain (Zig for libghostty, Rust for the app) means two different glibc assumptions coexist in one binary. Developers test on their own machine where everything matches.

**Consequences:** Packages that work on the build machine but segfault or refuse to start on user machines. Users file "broken package" issues.

**Prevention:**
- Build `.deb` packages on the oldest supported Ubuntu LTS (22.04) to get the lowest glibc baseline.
- Build `.rpm` packages on Fedora N-1 or Rocky/Alma container for RHEL targets.
- Use `ldd` + `readelf --needed` on the final binary to auto-detect all shared library dependencies and their minimum versions.
- Pin the Zig glibc target explicitly if building on a newer host: `-Dtarget=x86_64-linux-gnu.2.35`.
- For AppImage: build on Ubuntu 22.04 (or even 20.04) to minimize glibc floor.
- Run `strings cmux-app | grep GLIBC_ | sort -V | tail -1` to verify maximum glibc version required.

**Detection:** Run the package on a clean Ubuntu 22.04 Docker container. If it fails, the glibc baseline is too high.

**Phase to address:** First -- this determines the CI build environment for all packaging formats.

---

### Pitfall 2: GTK4 Runtime Resources Missing from Packages (Icons, Schemas, Pixbuf Loaders)

**What goes wrong:** The binary launches but shows no icons, wrong themes, or crashes with "GSettings schema not found" / "No GdkPixbuf loader for format PNG" errors. GTK4 apps require runtime resources beyond `.so` files: GLib compiled schemas, GdkPixbuf loader cache, Adwaita icon theme, GIO modules.

**Why it happens:** `cargo build` produces binaries but knows nothing about GTK4 runtime resources. Developer machines have everything installed system-wide. Package builders forget to declare dependencies or bundle resources.

**Consequences:** App launches with broken UI, missing icons, or hard crashes on minimal server or container installs.

**Prevention:**
- `.deb`: Depend on `libgtk-4-1`, `adwaita-icon-theme`, `gsettings-desktop-schemas`, `shared-mime-info`. Use `dpkg-shlibdeps` to auto-discover shared library deps.
- `.rpm`: Depend on `gtk4`, `adwaita-icon-theme`, `gsettings-desktop-schemas`.
- AppImage: Use `linuxdeploy-plugin-gtk` to bundle icons, schemas, and pixbuf loaders. Set in AppRun wrapper: `GSETTINGS_SCHEMA_DIR`, `GDK_PIXBUF_MODULE_FILE`, `GIO_MODULE_DIR`, `GTK_PATH`. Run `glib-compile-schemas` and `gdk-pixbuf-query-loaders --update-cache` during AppImage assembly.
- Flatpak: Use `org.gnome.Platform` runtime (not bare Freedesktop) which includes all GTK4 resources out of the box.

**Detection:** Launch the package in a minimal Docker container with no desktop environment. Check stderr for GLib/GdkPixbuf warnings.

**Phase to address:** Every format -- each has a different mechanism (system deps vs bundling vs runtime).

---

### Pitfall 3: Flatpak GPU/OpenGL Sandbox Blocks Ghostty Rendering

**What goes wrong:** cmux requires hardware OpenGL for Ghostty's GPU-accelerated terminal rendering via `GtkGLArea`. Inside the Flatpak sandbox, the app gets software rendering (llvmpipe) instead of hardware GPU -- resulting in <10fps scrolling, 100% CPU, and an unusable terminal.

**Why it happens:** Flatpak sandboxes device access by default. Without `--device=dri`, the GPU is inaccessible. Mesa driver version mismatches between the Flatpak runtime and host kernel compound the issue. NVIDIA users face a stricter constraint: userspace driver must exactly match the kernel module version.

**Consequences:** Terminal is unusable (software rendering) or crashes with EGL/GL initialization errors. Users on NVIDIA hit driver version mismatches. This is a showstopper for a terminal emulator.

**Prevention:**
- Add `--device=dri` to Flatpak finish-args (mandatory).
- Add `--socket=fallback-x11 --socket=wayland` for display server access.
- Use `org.gnome.Platform` runtime version 24.08+ for current Mesa.
- Include GL extensions in manifest: `org.freedesktop.Platform.GL.default` and conditionally `org.freedesktop.Platform.GL.nvidia-*`.
- Add a startup diagnostic that logs the GL renderer string (`glGetString(GL_RENDERER)`) to help users detect software fallback.
- Test on both Intel/AMD (Mesa) and NVIDIA systems before publishing.

**Detection:** `flatpak run --command=glxinfo your.app.id | grep "OpenGL renderer"`. If it says "llvmpipe", GPU is broken.

**Phase to address:** Flatpak packaging phase. Must be a pass/fail gate.

---

### Pitfall 4: Zig Cache Hash in build.rs Breaks CI Builds

**What goes wrong:** The current `build.rs` hardcodes a specific `.zig-cache` hash path for `simdutf.o`:
```rust
println!("cargo:rustc-link-arg={}/ghostty/.zig-cache/o/d36eec1e644b07f1d97ac6098a9555ba/simdutf.o", manifest_dir);
```
This hash is content-addressed and changes between Zig versions, clean builds, or different machines. CI builds from a clean checkout produce a different hash, and the build fails with "file not found."

**Why it happens:** Zig's build system uses content-addressed caching. The path worked on the developer's machine and was committed as-is.

**Consequences:** CI builds fail while developer machines succeed. Changing Zig versions breaks all builds until someone updates the hash manually.

**Prevention:**
- Replace the hardcoded hash path with a dynamic lookup: `find ghostty/.zig-cache -name simdutf.o -print -quit` in build.rs or setup script.
- Or: copy simdutf.o to a stable path (`ghostty/zig-out/lib/simdutf.o`) in the setup script after building libghostty.
- Pin Zig version exactly (0.15.2) in all CI environments.
- Cache `ghostty/zig-out/` and `ghostty/.zig-cache/` in CI, keyed on ghostty submodule SHA + Zig version.

**Detection:** Clean-checkout CI build fails while local builds succeed. The error message references a `.zig-cache/o/<hash>/` path that doesn't exist.

**Phase to address:** CI setup -- must fix before any packaging workflow works.

---

### Pitfall 5: AppImage Bundles glibc and Segfaults Everywhere

**What goes wrong:** The AppImage either (a) bundles glibc, which conflicts with the host's `ld-linux.so` and causes immediate segfaults, or (b) doesn't bundle glibc but was built on a newer distro, so older hosts lack required glibc symbols.

**Why it happens:** glibc is special: the dynamic linker (`ld-linux-x86-64.so.2`) must match the kernel. You cannot bundle it. But building on a new distro means your binary needs a newer glibc than old distros provide.

**Consequences:** AppImage either segfaults immediately (bundled glibc) or fails with "version GLIBC_2.XX not found" (too-new build host).

**Prevention:**
- NEVER bundle `libc.so`, `libpthread.so`, `ld-linux*.so`, `libdl.so`, `libm.so`, or `librt.so` in the AppImage.
- Use `linuxdeploy --exclude-library` for all glibc components.
- Build on the oldest supported distro (Ubuntu 22.04 container minimum, 20.04 if GTK4 packages are available).
- Verify with `strings cmux-app | grep GLIBC_ | sort -V` that the maximum required glibc matches the target baseline.

**Detection:** Run the AppImage on an Ubuntu 22.04 Docker container. If it fails, either glibc is bundled or the baseline is too high.

**Phase to address:** AppImage packaging phase.

---

## Moderate Pitfalls

### Pitfall 6: Go Binary (cmuxd-remote) Omitted from Packages

**What goes wrong:** The `cmuxd-remote` Go binary (required for SSH remote workspaces) is built separately from the Rust app. Packaging scripts forget to include it, or build it for the wrong architecture, or place it in a non-discoverable path.

**Why it happens:** Multi-language build (Rust + Go) means the package tooling doesn't automatically know about the Go binary. `cargo-deb` only packages Cargo outputs.

**Prevention:**
- Add explicit `cmuxd-remote` build step to every packaging script, before Rust build.
- Build with `CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o cmuxd-remote ./cmd/` for a fully static, architecture-explicit binary.
- Install to `/usr/lib/cmux/cmuxd-remote` (not in PATH -- it's an internal helper).
- Include in all package file lists: `.deb` postinst, `.rpm` spec %files, AppImage AppDir, Flatpak manifest.
- Verify with: install package, run `cmux ssh-connect` -- if cmuxd-remote is missing, SSH features fail.

**Detection:** Install package on a clean system, check if `/usr/lib/cmux/cmuxd-remote` exists and is executable.

**Phase to address:** Every format. Add to a shared checklist.

---

### Pitfall 7: Gitea Actions Silently Ignores concurrency Groups

**What goes wrong:** The existing GitHub CI uses `concurrency` groups with `cancel-in-progress` to avoid duplicate builds. Gitea Actions accepts this YAML without error but silently ignores it. Multiple CI runs for the same branch pile up, wasting runner resources and producing confusing overlapping results.

**Why it happens:** Gitea Actions is based on nektos/act and documents this as "not implemented." The YAML parses fine -- no error, no warning.

**Prevention:**
- Accept that concurrent runs will happen. Remove `concurrency` blocks to avoid false confidence.
- Design workflows to be idempotent (same inputs always produce same outputs).
- For expensive builds (Zig + Rust + Go = 15-30 min), consider simple locking: check if a release artifact already exists before rebuilding.
- Keep build times short so concurrent runs finish quickly.

**Detection:** Push two commits quickly to the same branch. Both CI runs execute fully instead of one cancelling the other.

**Phase to address:** Gitea CI setup (first workflow file).

---

### Pitfall 8: Gitea Token Cannot Publish to Package Registry

**What goes wrong:** The CI workflow tries to publish `.deb`/`.rpm` packages to the Gitea package registry using the auto-generated `GITEA_TOKEN`, but upload fails with 403/401.

**Why it happens:** The auto-generated `GITEA_TOKEN` in Gitea Actions does not have package registry write permissions. This is a documented limitation.

**Prevention:**
- Create a Personal Access Token (PAT) with package write permissions.
- Store as repository secret (e.g., `PACKAGE_PUBLISH_TOKEN`).
- Use this PAT in publish steps: `curl -H "Authorization: token ${{ secrets.PACKAGE_PUBLISH_TOKEN }}"`.
- Alternative: publish only to release assets (which the auto token CAN handle) and skip the package registry entirely.

**Detection:** CI publish step fails with auth errors despite other steps succeeding.

**Phase to address:** CI publish workflow. Decide early: release assets vs. package registry.

---

### Pitfall 9: Flatpak Offline Build Breaks Zig/Go Dependency Downloads

**What goes wrong:** Flatpak builds are sandboxed with NO network access during the build phase (mandatory for Flathub, good practice regardless). Zig's build system wants to fetch packages from the Zig package index. Go modules want to fetch from proxy.golang.org. Both fail silently or with cryptic errors.

**Why it happens:** Flatpak's security model requires all sources to be declared upfront in the manifest. Multi-toolchain projects (Rust + Zig + Go) multiply the problem because each has its own dependency resolution mechanism.

**Consequences:** Flatpak build fails with network errors or missing dependency errors. This is the single hardest packaging format for this project.

**Prevention:**
- **Zig (libghostty):** Pre-build `libghostty.a` outside the Flatpak sandbox and include it as a pre-built source artifact in the manifest (`type: file`). Do NOT attempt to run `zig build` inside the Flatpak builder.
- **Go (cmuxd-remote):** Either `go mod vendor` and include the vendor directory, or use `flatpak-go-mod-generator` to produce a sources JSON file. Or pre-build the static Go binary and include it like libghostty.
- **Rust:** Use `flatpak-cargo-generator` to produce a Cargo sources file, or `cargo vendor` + include the vendor directory.
- Simplest approach: pre-build ALL binaries (cmux-app, cmux, cmuxd-remote) outside Flatpak and just package them in the manifest. This sidesteps all toolchain issues.

**Detection:** Run the Flatpak build in a sandbox (`flatpak-builder --disable-rofiles-fuse`). Any network access attempt will fail.

**Phase to address:** Flatpak phase. Consider making Flatpak the last format due to complexity.

---

### Pitfall 10: RPM Built with Dead cargo-rpm Crate

**What goes wrong:** Developer reaches for `cargo-rpm` to build RPM packages, but it has been unmaintained since 2022 and fails with newer Rust toolchains.

**Prevention:**
- Use `cargo-generate-rpm` (actively maintained) instead.
- Or skip Cargo plugins entirely and use shell scripts with `rpmbuild` directly. For a multi-binary package (cmux-app, cmux CLI, cmuxd-remote) with non-Cargo assets (.desktop file, icons, metainfo), raw `rpmbuild` with a `.spec` file gives full control and is more transparent than fighting a Cargo plugin.
- Same reasoning applies to `.deb`: `cargo-deb` works but for complex packages with multiple binaries from different languages, a shell script calling `dpkg-deb` may be simpler.

**Detection:** `cargo install cargo-rpm` fails or produces deprecation warnings.

**Phase to address:** RPM packaging phase.

---

### Pitfall 11: Package Misses CLI Binary or Desktop Integration Files

**What goes wrong:** The package installs `cmux-app` (GTK4 application) but forgets the `cmux` CLI binary, or omits the `.desktop` file / icon / metainfo XML. Without a `.desktop` file, the app doesn't appear in application menus. Without metainfo, AppStream-based software centers (GNOME Software, KDE Discover) cannot display it.

**Why it happens:** Cargo workspace produces two binaries (`cmux-app`, `cmux`). Plus Go produces `cmuxd-remote`. Package tooling defaults to packaging only the primary binary. Desktop integration files must be created and installed manually.

**Prevention:**
- Create and maintain these files in the repo:
  - `data/com.cmux.cmux.desktop` (FreeDesktop .desktop file)
  - `data/com.cmux.cmux.metainfo.xml` (AppStream metainfo)
  - `data/icons/hicolor/*/apps/com.cmux.cmux.png` (app icon at 48, 128, 256px)
- Package install paths:
  - `/usr/bin/cmux-app` -- GTK4 app
  - `/usr/bin/cmux` -- CLI
  - `/usr/lib/cmux/cmuxd-remote` -- SSH remote daemon (not in PATH)
  - `/usr/share/applications/com.cmux.cmux.desktop`
  - `/usr/share/icons/hicolor/*/apps/com.cmux.cmux.png`
  - `/usr/share/metainfo/com.cmux.cmux.metainfo.xml`
- Verify with `dpkg -L cmux` / `rpm -ql cmux` that all files are present.
- Verify with `desktop-file-validate` and `appstream-util validate` in CI.

**Detection:** Install package, check if `cmux` CLI is in PATH and app appears in desktop menu.

**Phase to address:** All formats. Create the data files first, then reference them from every packaging script.

---

### Pitfall 12: Flatpak Theme/Icon Mismatch Confuses Users

**What goes wrong:** Users complain cmux looks wrong or doesn't match their desktop theme inside Flatpak. GTK4 theme support in Flatpak is severely limited -- custom themes are not supported, only Adwaita and themes packaged as Flatpak extensions.

**Why it happens:** Flatpak's sandbox blocks `/usr/share/themes`. GTK4 dropped the ability to load CSS from arbitrary host paths. This is a Flatpak platform limitation, not a bug.

**Prevention:**
- Use default Adwaita theme (this is the expected Flatpak behavior for GTK4 apps).
- Ensure the app looks good with both light and dark Adwaita variants (respect `prefer-dark` setting).
- Do NOT add `--filesystem=~/.themes` -- Flathub will reject it and it doesn't work well for GTK4 anyway.
- Document in the Flatpak description that theming follows system Adwaita preference.
- Add `--system-talk-name=org.freedesktop.portal.Settings` for portal-based dark mode detection.

**Detection:** Install Flatpak, switch system to dark mode -- app should follow. Switch to a custom GTK theme -- app should remain Adwaita (expected).

**Phase to address:** Flatpak phase.

---

## Minor Pitfalls

### Pitfall 13: Gitea Runner OOM During Zig Build

**What goes wrong:** Building libghostty.a with Zig is memory-intensive (4-8GB RAM). Self-hosted Gitea runners on small VMs get OOM-killed, producing cryptic "killed" errors with no useful diagnostic.

**Prevention:**
- Allocate at least 8GB RAM and 4 CPU cores to the Gitea runner.
- Set `timeout-minutes: 45` on the build job (Zig + Rust + Go can take 20-30 min).
- Cache `ghostty/zig-out/` and `ghostty/.zig-cache/` between runs, keyed on submodule SHA + Zig version.
- Consider pre-building libghostty.a in a separate workflow and caching the artifact (similar to existing `build-ghosttykit.yml` pattern).

**Detection:** CI job killed with signal 9 (OOM) during Zig build step. Check `dmesg` on the runner host.

**Phase to address:** CI infrastructure setup.

---

### Pitfall 14: Gitea Runner Labels Don't Support AND-Conditions

**What goes wrong:** Workflow specifies `runs-on: [ubuntu-latest, large]` expecting a runner matching ALL labels. Gitea interprets labels differently from GitHub.

**Prevention:**
- Use simple single labels: `runs-on: ubuntu-latest` or `runs-on: linux-builder`.
- Create specific runner labels for each machine type.

**Detection:** Jobs run on wrong runner or never get picked up.

**Phase to address:** CI setup.

---

### Pitfall 15: AppImage Missing GL/Vulkan Drivers

**What goes wrong:** The AppImage bundles Mesa but the bundled version doesn't match the host kernel's DRM driver, or bundled Mesa lacks the user's GPU driver (e.g., `radeonsi`, `iris`, `nouveau`). Result: software rendering or crash.

**Prevention:**
- Do NOT bundle Mesa/libGL in the AppImage. Let the host provide GPU drivers.
- Only bundle application-level libraries, not system graphics stack.
- Set `LD_LIBRARY_PATH` in AppRun to prefer host GL libraries over anything bundled.
- This means AppImage requires the host to have OpenGL drivers installed -- document this as a system requirement.

**Detection:** AppImage shows software rendering on a system with working GPU acceleration for native apps.

**Phase to address:** AppImage phase.

---

### Pitfall 16: Gitea Actions Workflow Syntax Differences Cause Silent Failures

**What goes wrong:** Copying GitHub Actions workflows to Gitea with minor unsupported features causes silent misbehavior, not errors. Beyond `concurrency`, these are silently ignored: `problem-matchers`, `success()`/`failure()`/`cancelled()` conditions (only `always()` works), complex `runs-on` label expressions, `permissions` blocks.

**Prevention:**
- Write Gitea workflows from scratch, not by copying GitHub ones.
- Use only `if: always()` or simple boolean expressions for conditionals.
- Test each workflow step individually when setting up CI.
- Reference the official comparison doc: https://docs.gitea.com/usage/actions/comparison

**Detection:** Workflows run but skip steps that should have run, or run steps that should have been skipped.

**Phase to address:** CI setup.

---

### Pitfall 17: stubs.o and glad.o Not Reproduced in CI

**What goes wrong:** The current `build.rs` links `stubs.o` and `glad.o` from the project root. These are pre-compiled object files that may have been built on the developer's machine. If they're not in git or not rebuilt in CI for the target architecture, the link step fails or produces a binary with wrong-architecture object files.

**Prevention:**
- Ensure `stubs.o` and `glad.o` are either checked into git OR built from source in the CI/packaging scripts.
- If building from source: compile `ghostty/vendor/glad/src/gl.c` to `glad.o` in the setup script.
- Verify object file architecture matches target: `file stubs.o` should show `x86-64` (or `aarch64` for ARM).

**Detection:** Link errors referencing undefined symbols from stubs.o or glad.o in CI.

**Phase to address:** CI setup / build script consolidation.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| CI infrastructure | Runner OOM (#13), Zig cache hash (#4), stubs.o/glad.o (#17) | 8GB+ runner, dynamic simdutf.o lookup, build .o files from source |
| Build scripts | glibc baseline (#1), missing Go binary (#6) | Build on oldest supported distro, explicit multi-binary build steps |
| .deb packaging | Missing CLI or desktop files (#11), wrong deps (#2) | dpkg-shlibdeps, explicit file list, desktop-file-validate |
| .rpm packaging | Dead cargo-rpm (#10), same file/dep issues (#11, #2) | Use cargo-generate-rpm or raw rpmbuild |
| AppImage | Bundle glibc (#5), missing GTK4 resources (#2), GL drivers (#15) | Never bundle glibc, use linuxdeploy-plugin-gtk, don't bundle Mesa |
| Flatpak | GPU broken (#3), offline build (#9), theme issues (#12) | --device=dri, pre-build all binaries, accept Adwaita theming |
| Gitea CI workflows | No concurrency (#7), token auth (#8), silent syntax diffs (#16) | Design idempotent, use PAT, write from scratch |
| Gitea publishing | Token cannot publish packages (#8) | Use PAT or publish to release assets only |

---

## Sources

- Gitea Actions vs GitHub Actions: https://docs.gitea.com/usage/actions/comparison
- Gitea Actions FAQ (token limitations): https://docs.gitea.com/usage/actions/faq
- Flatpak desktop integration: https://docs.flatpak.org/en/latest/desktop-integration.html
- Flatpak GTK4 theme limitation: https://github.com/flatpak/flatpak/issues/4605
- Flatpak GPU driver issues: https://github.com/flatpak/flatpak/issues/3673
- linuxdeploy GTK plugin: https://github.com/linuxdeploy/linuxdeploy-plugin-gtk
- AppImage GTK bundling: https://github.com/AppImage/AppImageKit/wiki/Bundling-GTK3-apps
- AppImage GTK4/libadwaita bundling: https://github.com/orgs/AppImage/discussions/1374
- Gaphor AppImage GTK4 migration: https://github.com/gaphor/gaphor/pull/1857
- AppImage bundling errors (pixbuf loaders): https://github.com/AppImageCrafters/appimage-builder/issues/243
- cargo-generate-rpm: https://crates.io/crates/cargo-generate-rpm
- Zig cross-compilation: https://zig.guide/build-system/cross-compilation/
- Ghostty PACKAGING.md: https://github.com/ghostty-org/ghostty/blob/main/PACKAGING.md
- Ghostty libghostty announcement: https://mitchellh.com/writing/libghostty-is-coming
- Gitea act_runner docs: https://docs.gitea.com/usage/actions/act-runner
- Gitea Actions caching: https://about.gitea.com/resources/tutorials/enable-gitea-actions-cache-to-accelerate-cicd
- Local codebase: `build.rs`, `Cargo.toml`, `.github/workflows/ci.yml`, `daemon/remote/go.mod`

---
*Pitfalls research for: cmux Linux v1.1 Packaging & Distribution*
*Researched: 2026-03-28*
