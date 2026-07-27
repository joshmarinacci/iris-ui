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
