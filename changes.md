## 2026-08-05

Fixed crash/panic risks from code review issues #1 and #2:

- `toggle_group.rs` `input_toggle_group` / `draw_toggle_group`: added `is_empty()` early-return guard before `bounds.size.w / items.len()` division to prevent divide-by-zero on an empty items list.
- `list_view.rs` `TextAction::Enter` path: replaced unchecked `items[state.selected]` with `items.get(state.selected)` so a stale `selected` index (e.g. after external truncation) cannot panic.
- Added regression tests: `test_toggle_group_empty_no_panic` and `test_list_view_enter_with_stale_selected_no_panic`.

## 2026-08-04 (2)

Fixed remaining crash paths in `list_view.rs` when the list is empty or has zero-height cells:

- `input_list` / `draw_list`: guard against `cell_height <= 0` (can happen when the view has no height yet) in addition to the empty-items guard added previously.
- `TextAction::Enter`: wrapped in `!state.items.is_empty()` check to prevent an out-of-bounds index on an empty vec.

## 2026-08-04

Added inline comments to `test_cliprect_nested` explaining how nested parent offsets accumulate into scene coordinates and what each assertion proves (`src/lib.rs`).

Added `test_cliprect_toggle_group_in_panel` — verifies that clicking a toggle group inside a panel marks only the toggle group's global bounds dirty, not the panel or full scene (`src/lib.rs`). Also adds `make_panel` and `make_toggle_group` imports to the test module.

Fixed divide-by-zero panic in `list_view.rs` when the list has no items: `input_list` and `draw_list` now return early if `state.items.is_empty()`.

## 2026-08-03

Fixed `cargo test` failures caused by optional crates (`test-log`, `embedded_graphics_simulator`, `env_logger`) being used without feature gates:

- `src/layouts.rs`, `src/lib.rs`, `src/tabbed_panel.rs`, `src/scene.rs`: Added `#[cfg(any(feature = "std", feature = "headless"))]` to test modules that depend on `test_log` or cross-reference helpers from other feature-gated test modules.
- `Cargo.toml`: Added `[[example]]` entry for `custom_view` with `required-features = ["std"]` so it is not compiled during a plain `cargo test`.

`cargo test` (no features) now runs 10 tests; `cargo test --features std` runs all 25.

## 2026-07-27 13:30

Added `examples/orientation.rs` — demonstrates display orientation using an inline `RotatedDisplay<D>` wrapper.

`embedded-graphics-transform 0.1.0` targets `embedded-graphics-core 0.3` (embedded-graphics 0.7.x) and is incompatible with this project's 0.8.x (core 0.4). Instead, `RotatedDisplay` is implemented directly in the example using the same coordinate transforms the crate uses:
- Rotate0: identity
- Rotate90: `(phys_w-1-y, x)`
- Rotate180: `(phys_w-1-x, phys_h-1-y)`
- Rotate270: `(y, phys_h-1-x)`

The wrapper implements `DrawTarget` and `Dimensions` so that `draw_content` receives a canvas whose `bounding_box()` reflects the logical (post-rotation) size, making the scene automatically adapt between landscape (0°/180°) and portrait (90°/270°).

Press **R** to cycle through all four rotations.

```
cargo run --example orientation --features std
```

## 2026-07-27 12:35

Made `EmbeddedDrawingContext` generic over display color type (`src/device.rs`).

Added `pub trait FromRgb565: PixelColor` with implementations for `Rgb565` (identity) and `BinaryColor` (black→Off, anything else→On). All drawing structs and functions (`EmbeddedDrawingContext`, `ScaledDisplay`, `draw_ttf_glyphs`) now use `T: DrawTarget` with `T::Color: FromRgb565` instead of the hard-coded `T: DrawTarget<Color = Rgb565>`. Colors from the UI layer (always `Rgb565`) are converted at the draw boundary via `T::Color::from_rgb565(...)`.

To use a `BinaryColor` display:
```rust
let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(size);
let mut ctx = EmbeddedDrawingContext::new(&mut display);
```

## 2026-07-27 12:10

Fixed TTF text centering in `ctx.text()` (`src/device.rs`). The `text()` method is used by `draw_centered_text`, which buttons call with `bounds.center()` as the position.

- **Horizontal**: The TTF branch was ignoring `style.halign` and starting the cursor at `position.x` directly, so all text drew from the center point rightward. Now `Align::Center` offsets the cursor left by `(total_w + first_xmin) / 2` so visible glyphs are centered on the position.
- **Vertical**: The TTF branch passed `position.y` directly as the baseline, placing text below center. Now `Align::Center` uses `font.horizontal_line_metrics` to compute `baseline_y = position.y + (ascent + descent) / 2`, which aligns the visual midpoint of the text with the given position (falls back to `size * 0.25` offset if metrics are unavailable).

## 2026-07-27 18:00

Added `--scale=N` command-line option to the simulator example. The scale defaults to 2 if not specified.

```
cargo run --example simulator --features std -- --scale=1
cargo run --example simulator --features std -- --scale=3
```

## 2026-07-27 17:45

Fixed compile error in `examples/simulator.rs`: `fontdue::Font::from_bytes` expects `impl Deref<Target=[u8]>`; passing `&Vec<u8>` resolves to `Vec<u8>`, not `[u8]`. Fixed by passing `bytes.as_slice()`.

## 2026-07-27 17:30

Added TTF font option to the simulator example and documented font usage in README.

- `examples/simulator.rs`: Added `get_ttf_font()` helper (gated behind `#[cfg(feature = "ttf")]`) that loads a system font at runtime (Geneva on macOS, DejaVu Sans on Linux) using `OnceLock<Option<fontdue::Font>>`. Added a **TTF** button to the font-size toolbar (also `#[cfg(feature = "ttf")]`) and a `"font-ttf"` command handler in `handle_events`. Run with `cargo run --example simulator --features std,ttf`.
- `README.md`: Replaced the Themes section with updated field names (`standard`, `panel`, `selected`, `accented`). Added a **Fonts** subsection explaining `FontKind::Bitmap` vs `FontKind::TrueType`, with std (`OnceLock`) and no_std (`MaybeUninit`) usage examples.

## 2026-07-27 17:00

Added optional TrueType font support via `fontdue`, gated behind the `ttf` Cargo feature.

- `Cargo.toml`: Added `fontdue = { version = "0.9", optional = true, default-features = false }` and `ttf = ["fontdue"]` feature. The `default-features = false` enables fontdue's `no_std + alloc` mode, matching the rest of the crate.
- `src/font.rs` (new): Defines `FontKind` enum with `Bitmap(MonoFont<'static>)` and (when `ttf` feature enabled) `TrueType { font: &'static fontdue::Font, size: f32 }` variants. Implements `Copy`, `Clone`, and `Debug`. Provides metric helpers: `char_width()`, `char_height()`, `baseline()`, `str_width()`.
- `src/lib.rs`: Re-exports `FontKind`. `Theme.font` and `Theme.bold_font` changed from `MonoFont<'static>` to `FontKind`. Added `#[derive(Clone, Copy)]` to `Theme`. Updated `BW_THEME` constant.
- `src/gfx.rs`: `TextStyle.font` changed from `&'a MonoFont<'static>` to `FontKind` (by value). `draw_centered_text` updated accordingly.
- `src/util.rs`: `calc_size` and `calc_bounds` updated to take `FontKind` and use metric helpers.
- `src/device.rs`: `fill_text` dispatches on `FontKind`. TTF rendering uses `draw_ttf_glyphs` (alpha threshold >127). Fixed pre-existing bug where text centering metrics were hardcoded to `FONT_6X10` regardless of the active font.
- All widget files updated to pass `FontKind` by value and use `.char_width()`/`.char_height()`/`.str_width()` instead of `.character_size.*`.
- `examples/simulator.rs` and `examples/custom_view.rs`: Updated `Theme` construction to wrap font constants with `FontKind::Bitmap(...)`.

Developer usage: `iris-ui = { features = ["ttf"] }`, then construct `fontdue::Font` in a `static` and pass `FontKind::TrueType { font, size }` in the theme.

## 2026-07-27 16:10

Added integer scale factor support to `Scene` and `EmbeddedDrawingContext`.

- `src/geom.rs`: Added `Bounds::scaled(scale)` and `Point::scaled(scale)` helpers.
- `src/scene.rs`: Added `scale: u32` field (default 1), `Scene::new_with_scale(bounds, scale)` constructor, and `scene.scale()` getter. All layout, picking, and dirty tracking remain in logical coordinates.
- `src/device.rs`: Added `scale` field to `EmbeddedDrawingContext` and `new_with_scale()` constructor. Geometric primitives (rect, line) multiply coordinates by scale. Text uses a `ScaledDisplay` wrapper that turns each logical pixel into a `scale×scale` block, achieving true pixel-doubling of bitmap fonts.
- `examples/simulator.rs`: Wired scale into display size, drawing context, clip, and mouse input (divides physical coords by scale before hit-testing). Enabled scale=2 in `make_scene()`.

Default scale is 1, so all existing call sites (including the ESP target) are unchanged.

## 2026-07-27 15:30

- Upgraded `embedded-graphics-simulator` from 0.7.0 to 0.8.0, which pulls in `sdl2` 0.38.0.
- Fixes a panic ("trying to construct an enum from an invalid value 0x207") caused by newer SDL2 system library (2.28+) emitting event types that `sdl2` 0.37.0 did not recognize.
