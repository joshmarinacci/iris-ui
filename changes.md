## 2026-08-19 (doc screenshot generator)

Added a script to regenerate the screenshots referenced in `docs/layout.md` directly from the
Rust snippets next to them, instead of hand-capturing them by running `examples/layout.rs`
interactively (which silently goes stale whenever a snippet changes).

- `examples/gen_doc_screenshots.rs`: scans `docs/layout.md` for fenced ` ```rust ` blocks
  followed by a `![...](file.png)` reference, and generates `examples/doc_screenshots.rs` (a
  gitignored build artifact, not committed) containing one `fn scene_N() -> Scene<Rgb565>` per
  matched snippet plus a `main()` that renders each headlessly and saves the PNG.
- `scripts/gen_doc_screenshots.sh`: runs the two examples in sequence (`gen_doc_screenshots` to
  codegen, then `doc_screenshots --features headless` to render). Uses
  `embedded-graphics-simulator`'s `headless` feature (`SimulatorDisplay` + `to_rgb_output_image().save_png()`),
  so no SDL/window is needed.
- `Cargo.toml`: added a `[[example]] name = "gen_doc_screenshots"` entry; `doc_screenshots` is
  deliberately *not* declared there since an explicit `[[example]]` entry requires the file to
  exist at manifest-parse time, which would break every other `cargo` invocation before the file
  has been generated once.
- Verified the regenerated PNGs are byte-identical to the previously hand-captured ones, and that
  editing a single snippet's button label and rerunning only changes that one screenshot.

## 2026-08-19 (list_view press-then-commit)

`src/list_view.rs`: `input_list` now distinguishes `PointerDown` from `PointerUp`, same pattern as `toggle_group.rs`/`button.rs`.

- `ListState` gained a private `pressed: Option<usize>` field.
- `PointerDown` hit-tests which row the touch landed on (via a new `row_index_at()` helper shared by both handlers) and stores it in `pressed`; sets focus and marks dirty, does not change `selected`.
- `PointerUp` re-hit-tests the release point and only commits (updating `selected`, firing `OutputAction::Command`) if release lands on the same row that was pressed — dragging off the row before lifting cancels it.
- `draw_list` inverts fill/text for the currently pressed row (even if not yet `selected`), so the touch is visible before release commits it.

Existing tests (`test_list_view`, `test_list_view_grows_to_fill_parent_width`, `test_list_view_enter_with_stale_selected_no_panic`) needed no changes — `test_list_view` drives via `click_at` (synthesized down+up at the same point), so `pressed == Some(n)` always holds. Verified with `cargo test --features std` (53 passed).

## 2026-08-19

`src/toggle_group.rs`: `input_toggle_group` now distinguishes `PointerDown` from `PointerUp` instead of only reacting to `PointerUp`, mirroring `button.rs`'s press/release pattern.

- `SelectOneOfState` gained a private `pressed: Option<usize>` field.
- `PointerDown` hit-tests which segment the touch landed on (via a new `cell_index_at()` helper shared by both handlers) and stores it in `pressed`; this only sets focus and marks the view dirty, it does not change `selected`.
- `PointerUp` re-hit-tests the release point and only commits the selection (updating `selected` and returning `OutputAction::Command`) if the release lands on the same segment that was pressed — dragging off the widget before lifting cancels the tap, same as `button.rs`.
- `draw_toggle_group` now inverts a segment's fill/text while it is the pressed one (even if not yet `selected`), so the touch is visible before release commits it.

Existing tests (`test_toggle_group`, `test_toggle_group_empty_no_panic`), which drive the widget via `click_at` (a synthesized down+up at one point), needed no changes and still pass — `click_at`'s down and up share the same point, so `pressed == Some(n)` always holds. Verified with `cargo test --features std` (53 passed).

## 2026-08-16

Parameterized the entire toolkit over the pixel color type instead of hardcoding `Rgb565`, so it can drive displays in any `embedded_graphics::pixelcolor::PixelColor` mode (`Rgb888`, `Gray8`, `BinaryColor`, etc.), not just RGB565.

- `src/gfx.rs`: `TextStyle` and the `DrawingContext` trait gained a `C: PixelColor` parameter (`TextStyle<'a, C>`, `DrawingContext<C>`); every color-bearing method now takes `&C` instead of `&Rgb565`.
- `src/lib.rs`: `ViewStyle`, `Theme`, `DrawEvent`, `LayoutEvent`, `GuiEvent`, and the `DrawFn`/`LayoutFn`/`InputFn`/`Callback` type aliases all gained `<C: PixelColor>`. `BW_THEME` stays a concrete `Theme<Rgb565>` (the common default); build your own `Theme<C>` literal for other color types.
- `src/view.rs`: `View` → `View<C: PixelColor>`; `src/scene.rs`: `Scene` → `Scene<C: PixelColor>`, and every dispatch/layout/draw function (`click_at`, `pointer_down_at`, `pointer_up_at`, `event_at_focused`, `pick_at`, `draw_scene`, `layout_scene`, ...) gained the same parameter.
- `src/device.rs`: generalized the old `FromRgb565` trait into `FromColor<Src: PixelColor>`, with a blanket identity impl (`impl<C> FromColor<C> for C`) plus the existing `BinaryColor`-from-`Rgb565` threshold conversion. `EmbeddedDrawingContext<'a, T>` became `EmbeddedDrawingContext<'a, T, C = T::Color>`, converting the theme's logical color `C` into the display's native color `T::Color` via `T::Color: FromColor<C>`. This preserves the existing two-tier design (one `Theme<Rgb565>` can still drive both a full-color display and a 1-bit e-paper display via automatic conversion, as `examples/simulator.rs` does) while letting the logical color be anything.
- All widget constructors (`make_button`, `make_label`, `make_panel`, `make_list_view`, `make_text_input`, `make_toggle_button`, `make_toggle_group`, `make_tabbed_panel`, `make_grid_panel`) and layout functions became generic over `<C: PixelColor>`. Their bodies were untouched — they only ever touch color through `theme.*.fill`/`.text`, so genericity falls out for free.
- `src/grid.rs`: the one widget with a hardcoded color (`Rgb565::RED` for debug gridlines) now uses `theme.accented.fill` instead, removing its `embedded_graphics::pixelcolor` dependency entirely.
- `src/test.rs`: `MockDrawingContext`/`make_mock_theme()` were deliberately kept concrete at `Rgb565` (not genericized) — every test constructs a scene/theme through one of these, so Rust's local type inference resolves `C = Rgb565` throughout each test body for free. As a result almost no test code needed to change; only a handful of test *helper functions* that had `Scene`/`View`/`DrawEvent`/`LayoutEvent`/`GuiEvent` in their own signature (not just their body) needed an explicit `<Rgb565>` added, since function signatures aren't inferred.
- `examples/simulator.rs`, `examples/custom_view.rs`, `esptest/src/main.rs`: updated call sites accordingly (`Theme<Rgb565>` on the theme consts, `Scene<Rgb565>`/`View<Rgb565>` on a few helper signatures, `FromRgb565` → `FromColor<Rgb565>` bound in `run_loop`). Behavior is unchanged — verified the simulator's existing dual Rgb565/BinaryColor (e-paper) code path still type-checks against one shared `Theme<Rgb565>`.

## 2026-08-15

Replaced the single `InputEvent::Tap(Point)` event with `InputEvent::PointerDown(Point)` and `InputEvent::PointerUp(Point)`, so the toolkit can distinguish a press from a release (mouse down/up on desktop, touch down/up on embedded).

- `src/scene.rs`: added `pointer_down_at` / `pointer_up_at`, each hit-testing at the given point and dispatching the corresponding event to the topmost view, exactly like the old `click_at`. `click_at` is now a convenience wrapper that calls `pointer_down_at` then `pointer_up_at` at the same point (simulates a full click) and is unchanged for existing callers/tests.
- `src/button.rs`: `ButtonState` gained a `pressed: bool` field. `PointerDown` sets focus + `pressed = true` (no action yet); `PointerUp` clears `pressed` and only fires `OutputAction::Command` if the button was actually pressed first. The draw function now inverts fill/text color while pressed, so the button visibly depresses and un-depresses. Added a `tests` module covering press/release/no-op-release/click_at.
- `src/text_input.rs`, `src/list_view.rs`, `src/toggle_group.rs`: their `InputEvent::Tap` match arms were renamed to `InputEvent::PointerUp` (selection/focus already only made sense on release).
- `src/toggle_button.rs`: `input_toggle_button` previously reacted to *any* event type; now explicitly filters to `InputEvent::PointerUp` only, otherwise it would double-toggle (once for down, once for up) since `click_at` now dispatches both.
- `examples/simulator.rs`: `MouseButtonDown` now calls `pointer_down_at` (previously a no-op); `MouseButtonUp` calls `pointer_up_at` instead of `click_at`.
- `esptest/src/main.rs`: the touch poll loop had no down/up edge detection (it called `click_at` every ~100ms poll while a finger stayed down). Added a `last_touch_point` state so a touch-down fires `pointer_down_at` once, and release (no touch point reported) fires `pointer_up_at` once at the last known point.

## 2026-08-07

Added `set_focus_enabled(bool)` / `is_focus_enabled()` to `Scene` in `src/scene.rs`. When focus is disabled, `set_focused` is a no-op: `focused` stays `None`, no dirty rects are produced for focus transitions, no focus rings are drawn, and `event_at_focused` dispatches to nothing. Useful on e-paper touch screens where minimizing dirty rects is critical. Three new tests in `focus_disabled_tests` verify the flag prevents dirty rect expansion, that the default (enabled) behaviour is unchanged, and that the flag can be toggled at runtime.

## 2026-08-05 (8)

Fixed `layout_list` in `list_view.rs` ignoring `h_flex = Grow`:

`layout_list` set the view height based on item count but never touched the width, leaving it at the default 100px regardless of the flex setting. Added a check: when `h_flex == Grow`, width is set to `e.space.w` (the space offered by the parent), matching the pattern used by all other layout functions.

Added regression test `test_list_view_grows_to_fill_parent_width` which places a `Grow` list view inside a 320px-wide `layout_vbox` parent and asserts the list width equals 320px. The test failed before the fix and passes after.

## 2026-08-05 (7)

Fixed gap accounting bug in `layout_hbox` and `layout_vbox`:

Previously, `gap` was applied during the child positioning pass but was never deducted from the space offered to `Grow` children. With N total children and a gap G, the Grow children were collectively given `G × (N−1)` extra pixels, causing them to overflow the container silently.

Fix: before dividing remaining space among Grow children, subtract `gap × (total_children − 1)` in both `layout_hbox` (`avail_horizontal_space`) and `layout_vbox` (`vert_leftover`).

Added two regression tests (`test_hbox_grow_child_accounts_for_gap`, `test_vbox_grow_child_accounts_for_gap`) that were written first to confirm the overflow, then pass after the fix.

## 2026-08-05 (6)

Minor fixes from code review issues #19–#20:

- **#19**: `Size::empty()` in `geom.rs` now returns `Size { w: 0, h: 0 }` instead of the magic sentinel `Size { w: -99, h: -99 }`. `is_empty()` treats any `w < 1 || h < 1` as empty, so zero is semantically identical but not misleading.
- **#20**: `TextInputState` cursor methods in `text_input.rs` now navigate by char boundary instead of by byte. `cursor_back` uses `char_indices().next_back()` to find the previous char's byte offset; `cursor_forward` uses `chars().next()` and `len_utf8()` to advance past the current char; `insert_char` advances by `key.len_utf8()`. This prevents panics on multi-byte char boundaries if non-ASCII content reaches the widget.

## 2026-08-05 (5)

Performance fixes from code review issues #17–#18:

- **#17**: `Scene::get_children_ids` now returns `&[ViewId]` instead of `Vec<ViewId>`, eliminating the heap allocation and full clone on every call. All call sites that need to mutate the scene after iterating children (layouts, draw, tabbed_panel, grid) collect to an owned `Vec` with `.to_vec()` first; pure read-only callers (pick, dump, filtered getter) iterate the slice directly.
- **#18**: `draw_view` now accepts an accumulated `offset: Point` and skips entire view subtrees whose global bounds do not intersect `scene.dirty_rect`. `Bounds::intersects` was added to `geom.rs` to support the check. When `dirty_rect.is_empty()` (full-redraw path), culling is disabled and all views are drawn. `draw_scene` passes `Point::zero()` as the initial offset.
- Added `Bounds::intersects` unit test in `geom.rs`.
- Added two unit tests in `scene.rs` (gated on `headless` feature): `test_dirty_rect_culls_non_overlapping_view` verifies that a view outside the dirty region is not drawn; `test_empty_dirty_rect_draws_all_views` verifies that an empty dirty_rect disables culling.

## 2026-08-05 (4)

API design fixes from code review issues #14–#16:

- **#14**: `make_label`, `make_header_label` (`label.rs`) and `make_text_input` (`text_input.rs`) now accept `name: &ViewId` instead of `name: &'static str`, matching every other widget constructor. Updated all call sites in `grid.rs` tests and `examples/simulator.rs`.
- **#15**: `ListState::new_with` and `SelectOneOfState::new_with` now return `Self` instead of `Box<dyn Any>`. Boxing moved to the call site in `make_list_view` and `make_toggle_group`. Removed now-unused `core::any::Any` imports from both files.
- **#16**: `PanelState` gains an explicit `impl Default` (`border_visible` cannot derive because its default is `true`). `SelectedState` gains `#[derive(Default)]`. Both keep their `new()` methods as thin delegates to `default()`.

## 2026-08-05 (3)

Idiomatic Rust cleanups from code review issues #7–#13:

- **#7**: `scene.rs` `set_focused` and `event_at_focused` — replaced `is_some()` + `unwrap()` pairs with `if let Some(...)`.
- **#8**: `scene.rs` `click_at` — changed `handlers: &Vec<Callback>` to `handlers: &[Callback]`.
- **#9**: `scene.rs` `get_children_ids_filtered` — replaced `.map(...).flatten()` with `.filter_map(...)`.
- **#10**: moved `&'static str → ViewId` conversion from an `impl Into` in `grid.rs` to a proper `impl From<&'static str> for ViewId` in `view.rs` where it applies unconditionally.
- **#11**: `grid.rs` — removed two empty no-op `if view.h_flex == Shrink {}` / `if view.v_flex == Shrink {}` branches; removed now-unused `Shrink` import.
- **#12**: `layouts.rs` — removed redundant `.clone()` calls on `Copy` types (`Insets`, `Flex`, `Size`) in `layout_vbox`, `layout_hbox`, and `layout_std_panel`.
- **#13**: `layouts.rs` `layout_vbox` fold — removed misleading `return` inside the closure (returns from the closure, not the function).

## 2026-08-05 (2)

Fixed correctness bugs from code review issues #3–#6:

- **#3 (already fixed)**: `layouts.rs` hbox flex space correctly uses `available_space.w` — carried over from the prior "fix hbox layout bug" commit. Updated the `test_hbox_fixed_width` assertion to match correct values (child2 width 120px, child3 x=160).
- **#4**: `scene.rs` `move_view_to_parent` now also calls `self.parents.insert(child, parent)` so `get_parent_for_view` returns the correct parent after a move. Added assertion to `parent_child` test.
- **#5**: `scene.rs` `remove_parent_and_children` is now recursive: replaces `remove_view(&kid)` with `remove_parent_and_children(&kid)` so grandchildren are fully cleaned up from all three maps. Added `remove_parent_and_children_cleans_grandchildren` regression test.
- **#6**: `geom.rs` `Point::scaled` and `Bounds::scaled` now use `scale.min(i32::MAX as u32) as i32` instead of `scale as i32` to avoid silent wrapping on large scale values.

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
