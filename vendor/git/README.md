# Vendored git dependencies (Optimised PDF)

PdfCraft v0.21.0 took these crates from storytold's GitHub repositories, pinned to exact
revisions. Optimised PDF keeps a copy of each here so that a build never needs those repositories.
They are path dependencies (`Cargo.toml`), unchanged apart from their manifests: each copy came from
`cargo vendor` (which writes a self-contained manifest), and dependencies between them were pointed
at their folders here.

| Folder | Repository | Revision |
|---|---|---|
| `ecolor`, `eframe`, `egui`, `egui-wgpu`, `egui-winit`, `egui_extras`, `egui_glow`, `egui_kittest`, `emath`, `epaint`, `epaint_default_fonts` | storytold/egui | `725a39fca940f77149ffbb9c58eef405a16997bc` |
| `winit` | storytold/winit | `a517b3846ee8457d84ee4975110bfda6797c44dc` |
| `craft-docpath`, `craft-fonts-build`, `craft-i18n`, `craft-keymap`, `craft-menu`, `craft-quit`, `craft-ui` | storytold/craft-libs | `787a7a0f972fd1d7a7618fdbeca3a5267a0407ff` |
| `craft-print` | storytold/craft-libs | `60ede99985ef1e5df87a2aca66ed538eb23dec99` |
| `craft-web-api`, `craft-web-host` (web build only) | storytold/craft-libs | `fd5e2feff6ea25bed371a99cc732b3968d621505` |
| `craft-winprint-gdi` | storytold/winprint.rs | `64e7afd4d1483512385012c340ecaaaf0f3beb7f` |
| `winprint` | storytold/winprint.rs | `bc06a7c48b768eb27a828c7986f6d2868b245ee0` |

Manifest changes:
- Dependencies between these crates point at their folders here (`path = "../<crate>"`).
- `winprint`: PdfCraft used two revisions, `5f80b9b` (crates/print) and `bc06a7c` (craft-print).
  Cargo takes one copy of a crate version from a path, so both now use `bc06a7c`, which is a later
  commit on the same history (it adds to `5f80b9b`; nothing is lost).
- Optional or test-only dependencies on crates not used here lost their paths: `eframe`'s
  `egui_inspection` and `egui_system_font_provider` (features `inspection`, `system_fonts`, off),
  and `craft-keymap`'s dev-dependency `craft-settings` (removed).

`vendor/winit` and `vendor/egui-wgpu` (one level up) are older copies kept by upstream as the record
of earlier patches; they are not used by the build.

Licences: each crate keeps its own licence files; all are MIT and/or Apache-2.0 (egui, winit,
craft-libs, winprint.rs), and the fonts in `epaint_default_fonts` are listed in `ATTRIBUTION.toml`.
