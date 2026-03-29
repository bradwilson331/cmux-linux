# Roadmap: cmux Linux Port

## Milestones

- [x] **v1.0 Linux Port MVP** -- Phases 1-10 (shipped 2026-03-28)
- [ ] **v1.1 Linux Packaging & Distribution** -- Phases 11-14 (in progress)

## Phases

<details>
<summary>v1.0 Linux Port MVP (Phases 1-10) -- SHIPPED 2026-03-28</summary>

- [x] Phase 1: Ghostty Foundation (9 plans)
- [x] Phase 2: Workspaces + Pane Splits (8 plans)
- [x] Phase 3: Socket API + Session Persistence (8 plans)
- [x] Phase 4: Notifications + HiDPI + SSH (7 plans)
- [x] Phase 5: Config + Distribution (2 plans)
- [x] Phase 6: Session Layout Restore + Surface Wiring (2 plans)
- [x] Phase 7: SSH Terminal I/O (4 plans)
- [x] Phase 7.1: SSH Workspace UI (1 plan)
- [x] Phase 8: Agent-Browser Integration (6 plans)
- [x] Phase 9: UI Buttons and Menus (3 plans)
- [x] Phase 10: CLI Socket Commands (2 plans)

Full details: [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md)

</details>

### v1.1 Linux Packaging & Distribution

- [x] **Phase 11: Desktop Integration & Dependency Detection** - Shared metadata files and runtime dep-detection that all packaging formats consume (completed 2026-03-29)
- [ ] **Phase 12: Native Packages (.deb + .rpm)** - Installable packages for Debian/Ubuntu and Fedora/RHEL
- [ ] **Phase 13: Portable Formats (AppImage + Flatpak)** - Self-contained distribution without package manager dependency
- [ ] **Phase 14: Build Automation & CI Pipeline** - Unified build script, GPG signing, and Gitea Actions for tag-triggered releases

## Phase Details

### Phase 11: Desktop Integration & Dependency Detection
**Goal**: All shared metadata files exist and validate correctly, so packaging phases can reference them without creating anything new
**Depends on**: Nothing (first phase of v1.1)
**Requirements**: META-01, META-02, META-03, META-04, META-05, BUILD-02
**Success Criteria** (what must be TRUE):
  1. `appstreamcli validate` passes on the metainfo XML with reverse-DNS ID `com.cmux_lx.terminal`
  2. PNG icons at 48px, 128px, 256px exist under hicolor icon theme directory structure
  3. Shell completions for bash, zsh, and fish are generated; man page renders via `man ./cmux.1`
  4. A dependency detection script maps ldd output of cmux-app to both Debian and Fedora package names
**Plans:** 3/3 plans complete

Plans:
- [x] 11-01-PLAN.md -- Desktop metadata (desktop entry, metainfo XML, icons, validation script)
- [x] 11-02-PLAN.md -- Shell completions and man page (clap_complete + clap_mangen generator)
- [ ] 11-03-PLAN.md -- Dependency detection script (ldd to Debian/Fedora package mapping)

### Phase 12: Native Packages (.deb + .rpm)
**Goal**: Users on Debian/Ubuntu and Fedora/RHEL can install cmux from a single package file with all dependencies resolved automatically
**Depends on**: Phase 11
**Requirements**: DEB-01, DEB-02, DEB-03, DEB-04, RPM-01, RPM-02, RPM-03
**Success Criteria** (what must be TRUE):
  1. `dpkg -i cmux.deb` installs successfully on Ubuntu 22.04+ and cmux launches from terminal
  2. `apt install -f` after dpkg resolves all runtime dependencies (GTK4, GL, fontconfig, freetype, oniguruma)
  3. `dnf install cmux.rpm` installs successfully on Fedora 38+ and cmux launches from terminal
  4. Both packages install cmux-app and cmux CLI to `/usr/bin/` and cmuxd-remote to `/usr/lib/cmux/`
**Plans:** 1/2 plans executed

Plans:
- [ ] 12-01-PLAN.md -- .deb packaging script and validation (build-deb.sh, validate-deb.sh)
- [x] 12-02-PLAN.md -- .rpm spec file, packaging script, and validation (cmux.spec, build-rpm.sh, validate-rpm.sh)

### Phase 13: Portable Formats (AppImage + Flatpak)
**Goal**: Users can run cmux without a package manager -- either as a download-and-run AppImage or via Flatpak with sandboxed GPU access
**Depends on**: Phase 11
**Requirements**: APPIMG-01, APPIMG-02, APPIMG-03, FLAT-01, FLAT-02, FLAT-03
**Success Criteria** (what must be TRUE):
  1. Downloaded AppImage is executable and runs on a clean Ubuntu 22.04 system without any cmux packages installed
  2. AppImage bundles GTK4 runtime resources (schemas, loaders, Adwaita theme) and renders the terminal correctly
  3. `flatpak install` from local bundle succeeds and cmux launches with GPU-accelerated rendering on Wayland
  4. Flatpak sandbox permissions allow display (Wayland/X11), GPU (DRI), network (SSH), filesystem, and desktop notifications
**Plans**: TBD

### Phase 14: Build Automation & CI Pipeline
**Goal**: A single script builds all package formats locally, and pushing a git tag triggers automated builds with publishing to Gitea
**Depends on**: Phase 12, Phase 13
**Requirements**: BUILD-01, BUILD-03, CI-01, CI-02, CI-03, CI-04
**Success Criteria** (what must be TRUE):
  1. `./packaging/build-all.sh` produces .deb, .rpm, AppImage, and Flatpak artifacts in one run
  2. Each format is individually selectable (e.g., `./packaging/build-all.sh deb`)
  3. Pushing a version tag to Gitea triggers a workflow that builds all formats and attaches them as release assets
  4. .deb and .rpm packages are published to Gitea package registry (accessible via apt/dnf repo configuration)
  5. act_runner is configured for host-mode execution with Zig, Rust, and Go toolchains available
**Plans**: TBD

## Progress

**Execution Order:**
Phases 11 first, then 12 and 13 can run in parallel, then 14 last.

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Ghostty Foundation | v1.0 | 9/9 | Complete | 2026-01-22 |
| 2. Workspaces + Pane Splits | v1.0 | 8/8 | Complete | 2026-02-01 |
| 3. Socket API + Session Persistence | v1.0 | 8/8 | Complete | 2026-02-10 |
| 4. Notifications + HiDPI + SSH | v1.0 | 7/7 | Complete | 2026-02-20 |
| 5. Config + Distribution | v1.0 | 2/2 | Complete | 2026-03-01 |
| 6. Session Layout + Surface Wiring | v1.0 | 2/2 | Complete | 2026-03-10 |
| 7. SSH Terminal I/O | v1.0 | 4/4 | Complete | 2026-03-20 |
| 7.1. SSH Workspace UI | v1.0 | 1/1 | Complete | 2026-03-22 |
| 8. Agent-Browser Integration | v1.0 | 6/6 | Complete | 2026-03-25 |
| 9. UI Buttons and Menus | v1.0 | 3/3 | Complete | 2026-03-27 |
| 10. CLI Socket Commands | v1.0 | 2/2 | Complete | 2026-03-28 |
| 11. Desktop Integration & Dep Detection | v1.1 | 2/3 | Complete    | 2026-03-29 |
| 12. Native Packages (.deb + .rpm) | v1.1 | 1/2 | In Progress|  |
| 13. Portable Formats (AppImage + Flatpak) | v1.1 | 0/0 | Not started | - |
| 14. Build Automation & CI Pipeline | v1.1 | 0/0 | Not started | - |
