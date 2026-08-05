# Code Review Findings

Issues ranked by severity. Check the box when implemented.

---

## Crash / Panic Risk

- [x] **1. `toggle_group.rs` lines 51, 71 — division by zero on empty items**
  Both `input_toggle_group` and `draw_toggle_group` compute
  `bounds.size.w / state.items.len()` with no guard for an empty list.
  Same bug that was just fixed in `list_view.rs`: add `if state.items.is_empty() { return; }`
  before the division in both functions.

- [x] **2. `list_view.rs` line 97 — latent panic in Enter-key path**
  The `!is_empty()` guard was added, but `items[state.selected]` is still an unchecked index.
  If the list is shortened externally after `selected` was set, this panics.
  Fix: use `state.items.get(state.selected)` and handle `None`.

---

## Correctness (wrong results, not panics)

- [x] **3. `layouts.rs` line 144 — hbox reads `.h` instead of `.w` for horizontal flex space**
  ```rust
  // current (wrong):
  let avail_horizontal_space = (available_space - padding).h - kids_sum;
  // correct:
  let avail_horizontal_space = available_space.w - kids_sum;
  ```
  Also double-subtracts `padding` (already removed from `available_space` on line 126).
  The existing test uses a square 200×200 scene so `w == h` and the bug is invisible.
  On any non-square display (e.g. 320×240) flex children get the wrong width.

- [x] **4. `scene.rs` lines 290–297 — `move_view_to_parent` doesn't update the `parents` map**
  The function pushes `child` into the new parent's `children` list but never calls
  `self.parents.insert(child.clone(), parent.clone())`.
  After a move, `get_parent_for_view(child)` still returns the old parent.
  The test only asserts `get_children_ids` length, not the parent side.

- [x] **5. `scene.rs` lines 299–306 — `remove_parent_and_children` leaks grandchildren**
  `self.remove_view(&kid)` only removes from `self.keys`; it does not touch `self.children`
  or `self.parents`. Any grandchildren of a removed child remain as orphaned entries in all
  three maps. Fix: replace the two inner calls with a recursive
  `self.remove_parent_and_children(&kid)`.

- [x] **6. `geom.rs` lines 138–139, 214–221 — silent `u32 → i32` cast in `scaled()`**
  `let s = scale as i32` wraps silently if `scale > i32::MAX`.
  Fix: use `scale.min(i32::MAX as u32) as i32` or change the parameter type to `i32`.

---

## Idiomatic Rust / Anti-patterns

- [x] **7. `scene.rs` lines 63–67, 370–373 — `is_some()` + `unwrap()` pairs**
  ```rust
  // current:
  if self.focused.is_some() {
      let fo = self.focused.as_ref().unwrap().clone();
  }
  // fix:
  if let Some(fo) = self.focused.clone() { ... }
  ```
  Triggers `clippy::unnecessary_unwrap`. Appears in `set_focused` and `event_at_focused`.

- [x] **8. `scene.rs` line 341 — `&Vec<Callback>` parameter should be `&[Callback]`**
  The function only iterates; `&[T]` is the idiomatic slice reference.
  Triggers `clippy::ptr_arg`.

- [x] **9. `scene.rs` lines 139–147 — `.map(...).flatten()` should be `.filter_map(...)`**
  `.map(f).flatten()` where `f` returns `Option` is exactly what `.filter_map(f)` does.
  Triggers `clippy::map_flatten`.

- [x] **10. `grid.rs` lines 191–195 — `Into` impl instead of `From`, placed in wrong module**
  ```rust
  // current (in grid.rs):
  impl Into<ViewId> for &'static str { ... }
  // fix (move to view.rs):
  impl From<&'static str> for ViewId { ... }
  ```
  The blanket impl provides `Into` automatically from `From`. Keeping it in `grid.rs` means
  it only applies when `grid` is in scope.

- [x] **11. `grid.rs` lines 154–158 — empty dead-code `if Shrink {}` branches**
  Two empty `if view.h_flex == Shrink {}` / `if view.v_flex == Shrink {}` blocks do nothing
  and should be removed.

- [x] **12. `layouts.rs` lines 18–20, 115–116, 207, 215 — `.clone()` on `Copy` types**
  `Insets`, `Flex`, `Size`, and `Bounds` all derive `Copy`. Calling `.clone()` on them is
  redundant and misleads readers. Remove the `.clone()` calls on these types throughout
  `layouts.rs`.

- [x] **13. `layouts.rs` lines 41–46 — `return` inside a `fold` closure**
  `return` inside a closure returns from the closure, not the outer function — it works but
  is unusual and misleading. Replace with a direct expression.

---

## API Design

- [x] **14. `label.rs`, `text_input.rs` — take `&'static str` instead of `&ViewId`**
  Every other widget constructor (`make_button`, `make_panel`, `make_toggle_group`, etc.)
  accepts `name: &ViewId`. These two can't accept runtime-generated names.
  Fix: change signatures to `name: &ViewId`.

- [x] **15. `list_view.rs` line 44, `toggle_group.rs` line 35 — constructors return `Box<dyn Any>`**
  `ListState::new_with` and `SelectOneOfState::new_with` erase their own type at construction.
  They should return `Self`; boxing can be done at the call site where
  `Option<Box<dyn Any>>` is needed.

- [x] **16. `panel.rs`, `toggle_button.rs` — `PanelState::new()` should implement `Default`**
  Both have a no-argument `new()` that creates canonical zero-state. These should use
  `#[derive(Default)]` or an explicit `impl Default` so they work with spread syntax and
  generic bounds.

---

## Performance

- [ ] **17. `scene.rs` lines 129–135 — `get_children_ids` clones the full `Vec` on every call**
  Called repeatedly per frame in layout, draw, and pick paths. Each clone heap-allocates
  all `ViewId` strings. Fix: return `&[ViewId]` (requires lifetime annotation) or at minimum
  use a `Cow`.

- [ ] **18. `scene.rs` lines 416–424 — `draw_scene` ignores `dirty_rect` for culling**
  `dirty_rect` is carefully maintained but `draw_scene` always clears and redraws the entire
  view tree on every dirty frame. For embedded displays where partial redraws matter,
  `draw_view` should skip subtrees whose global bounds don't intersect `dirty_rect`.

---

## Minor / Design

- [ ] **19. `geom.rs` line 11 — magic `-99` sentinel in `Size::empty()`**
  `Size { w: -99, h: -99 }` is indistinguishable from an arithmetic underflow bug.
  `is_empty()` treats any `w < 1 || h < 1` as empty, so `Size { w: 0, h: 0 }` works
  and is less surprising.

- [ ] **20. `text_input.rs` — byte cursor assumes ASCII, no enforcement**
  `cursor_forward`/`cursor_back` move by 1 byte. `String::insert`/`remove` panic on
  multi-byte character boundaries. `TypedAscii(u8)` keeps this safe today, but there is no
  compile-time or runtime guard. At minimum, add a comment documenting the ASCII-only
  assumption. Full fix: use `char_indices()` for cursor navigation.

---

*No `unsafe` code was found anywhere in the codebase.*
