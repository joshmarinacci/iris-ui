## 2026-07-27 15:30

- Upgraded `embedded-graphics-simulator` from 0.7.0 to 0.8.0, which pulls in `sdl2` 0.38.0.
- Fixes a panic ("trying to construct an enum from an invalid value 0x207") caused by newer SDL2 system library (2.28+) emitting event types that `sdl2` 0.37.0 did not recognize.
