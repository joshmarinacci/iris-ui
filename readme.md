# Iris-UI

![screenshot](resources/screenshot-002.png)

## What is This?

Iris a UI library for **no_std** embedded Rust. I currently have it running on the ESP32-S3
based [Lilygo T-Deck](https://github.com/Xinyuan-LilyGO/T-Deck/tree/master), but it should run on anything that uses
the [embedded_graphics traits](https://docs.rs/embedded-graphics/latest/embedded_graphics/). It focuses on bandwidth
limited devices, such as SPI displays.

## Features

* Incremental redrawing using layout and dirty rect tracking.
* Built in components for buttons, labels, text input, toggles, and panels.
* Theming with colors and fonts
* Scene graph to manage a tree of View structs
* Fast single pass layout algorithm

## Anti-Features

* **event loop:** To make it flexible, the lib **does not** impose its own event loop. Instead, the application should
  send events to the scene and then redraw in its own loop. See [Event Loop](#event-loop) below.
* **animation:** The library has no support for animation or transparency because those will perform horribly on
  bandwidth limited SPI displays.

## Usage

### Use as a crate

Add the crate `iris-ui` to your `Cargo.toml` file then `use iris_ui::*` in your code. Check out
the [example code](examples).

### Building and running locally

Build the library with `cargo build`.

Run the simulator example with `cargo run --example simulator --features std`. Note that the simulator needs
SDL2. [Install instructions](https://docs.rs/embedded-graphics-simulator/latest/embedded_graphics_simulator/).

Run the unit tests with `cargo test --features std`.

Regenerate the screenshots in [docs/layout.md](docs/layout.md) from their own code snippets with
`./scripts/gen_doc_screenshots.sh` (no SDL required — it renders headlessly).

## Views

Rather than using inheritance, which is a bad fit for Rust, every component / widget / control is an instance of the
`View` struct. Views are referenced by their `name` property so the names should be unique throughout your application.
All views several mandatory fields like name, visible, bounds, etc, as well as optional fields for state, input
handlers, layout, and drawing.

| Field     | Value                    | Description                                                                                       |
|-----------|--------------------------|---------------------------------------------------------------------------------------------------|
| `name`    | string                   | should be unique throughout your application.                                                     | 
| `title`   | string                   | Used by buttons as the display text                                                               |
| `bounds`  | Bounds (position & size) | should only be modified inside of the layout function                                             |
| `v_flex`  | grow, shrink or fixed    | indicates if the view wants itself grow, shrink, or have a fixed size in the vertical direction   |
| `h_flex`  | grow, shrink or fixed    | indicates if the view wants itself grow, shrink, or have a fixed size in the horizontal direction |
| `v_align` | start, center, or end    | indicates how the view wants to be aligned vertically                                             |
| `h_align` | start, center, or end    | indicates how the view wants to be aligned horizontally                                           |
| `visible` | bool                     |                                                                                                   |
| `state`   | Option\<Box\<dyn Any\>\> | optional object for the state of the view                                                         |
| `input`   | Option\<InputFn\>        | optional input handler function                                                                   |
| `layout`  | Option\<LayoutFn\>       | optional layout function                                                                          |
| `draw`    | Option\<DrawFn\>         | optional drawing function                                                                         |

`View`s are rendered using a `Theme` which can be customized for different colors and font sizes. Views carry their own
internal state using an optional `state` struct. Application state should remain outside the scene/view structure and be
handled by processing actions emitted from the scene when events happen.

Instead of implementing a trait you create components by allocating a `View` is with optional fields for functions to
handle input, state, layout, and drawing. This is the code that creates a button (as implemented in the library provided
`make_button`):

```rust
pub fn make_button(name: &ViewId, title: &str) -> View {
    View {
        name: name.clone(),
        title: title.to_string(),
        // the button will determine its own width
        h_flex: Intrinsic,
        // the button will determine its own height
        v_flex: Intrinsic,
        // on release, requested to be focused
        input: Some(|e| {
            if let EventType::PointerUp(_pt) = &e.event_type {
                e.scene.set_focused(e.target);
                return Some(Action::Generic);
            }
            None
        }),
        // size self based on the font and the title text
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(&e.target) {
                view.bounds.size = util::calc_size(e.theme.bold_font, &view.title);
            }
        }),
        // delegate drawing to a draw_button function
        draw: Some(draw_button),
        ..Default::default()
    }
}

fn draw_button(e: &mut DrawEvent) {
    e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
    e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
    if let Some(focused) = e.focused {
        if focused == &e.view.name {
            e.ctx.stroke_rect(&e.view.bounds.contract(2), &e.theme.fg);
        }
    }
    draw_centered_text(
        e.ctx,
        &e.view.title,
        &e.view.bounds,
        &e.theme.bold_font,
        &e.theme.fg,
    );
}
```

## Built In Views

Iris has built in views for buttons, labels, text input, lists, and more. To use them call the `make_*` function from
the list below, then customize it using `with_*` functions. ex: make a center aligned invisible button:
```rust
let button_id = ViewId::new("button1");
let mut button = make_button( & button_id, "My cool button")
    .with_h_align(Align::Center)
    .with_visible(false);
```

| name           | function           | description                                 |
|----------------|--------------------|---------------------------------------------|
| button         | make_button        | standard button                             |
| primary button | make_full_button   | with a command and primary color            |
| toggle button  | make_toggle_button | button with a selected state                |
| label          | make_label         | a plain label                               |
| header label   | make_header_label  | a bold label in the accent color            |
| scrolling list | make_list_view     | a scrolling list of items with one selected |
| panel          | make_panel         | a standard panel                            |
| tabbed panel   | make_tabbed_panel  | panel with several tabs                     |
| text input     | make_text_input    | single line text input                      |
| toggle group   | make_toggle_group  | group of exclusive toggle buttons           |





## Custom Views

All views are just instances of the `View` struct. To create a custom view build a View with custom `state`, `input`,
`layout`, and `draw` fields. This example creates a simple progress bar.

First create a struct to represent the internal state of the progress bar:

```rust
// struct for the state of the progress bar
struct ProgressState {
    value: f32,
}
```

Now make a function to return a view with custom attributes.

```rust
fn make_progress_bar(name: &ViewId) -> View {
    View {
        name: name.clone(),

        // set the state
        state: Some(Box::new(ProgressState {
            value: 0.0,
        })),

        // no input
        input: None,

        // fixed size layout
        layout: Some(|e| {
            if let Some(view) = e.scene.get_view_mut(e.target) {
                view.bounds.size = Size::new(100, 20);
            }
        }),

        // draw progress bar
        draw: Some(|e| {
            e.ctx.fill_rect(&e.view.bounds, &e.theme.bg);
            let full = e.view.bounds.size;
            // get the state to calculate the fill width
            if let Some(state) = e.view.get_state::<ProgressState>() {
                let w = (full.w as f32 * state.value) as i32;
                let bd2 = Bounds::new_from(e.view.bounds.position, Size::new(w, full.h));
                e.ctx.fill_rect(&bd2, &e.theme.selected_bg);
            }
            e.ctx.stroke_rect(&e.view.bounds, &e.theme.fg);
        }),

        // use defaults for the rest of the attributes
        ..Default::default()
    }
}
```

Now call the function to build the view and add it to your scene.

```rust
fn make_progressbar() {
    let progress_id = ViewId::new("progress_bar");
    scene.add_view_to_root(make_progress_bar(&progress_id));
}
```

When the state of progress needs to change, update the state inside of a `get_view_state()` call.

```rust
fn update_progressbar() {
    // update the progress bar every 100 msec
    if let Some(state) = scene.get_view_state::<ProgressState>(&progress_id) {
        state.value += 0.01;
        if state.value > 1.0 {
            state.value = 0.0;
        }
        scene.mark_dirty_view(&progress_id);
        sleep(Duration::from_millis(100));
    }
}
```

See the full example code in [examples/custom_view.rs](examples/custom_view.rs).

## Themes

`Theme` is a struct passed to every View's `draw` function. It stores the standard colors and fonts for drawing.
However, these are just guidelines. A view can feel free to ignore them and draw whatever it wants. The theme fields
should be used for:

* **standard**: fill and text colors for buttons, text inputs, and most interactive components.
* **panel**: fill and text colors for panels and containers — may differ from `standard` depending on the theme.
* **selected**: colors used to indicate a selected or focused state.
* **accented**: highlight color, used for primary buttons or decorations.
* **font**: the default font used for all text.
* **bold_font**: the bold variant of the current font. Used for button titles.

Each color group is a `ViewStyle<C> { fill: C, text: C }` where `C` is the color mode. See below for more information on Color Modes.

### Color modes

`Theme`, `View`, `Scene`, and every widget constructor are generic over a color type `C` — any
`embedded_graphics::pixelcolor::PixelColor` (`Rgb565`, `Rgb888`, `Gray8`, `BinaryColor`, etc.) can be used. Most
call sites never need to write `<C>` explicitly: it's inferred from whatever theme/display you construct the scene
with. `BW_THEME` is a ready-made `Theme<Rgb565>`; for other color types, build your own `Theme<C>` literal.

On the device side, `EmbeddedDrawingContext<'a, T, C>` wraps an `embedded_graphics::DrawTarget` `T` and converts your
theme's logical color `C` into the display's native color `T::Color` via the `FromColor<C>` trait (in `iris_ui::device`).
This is what lets one `Scene<Rgb565>` drive both a full-color display and a 1-bit e-paper display side by side — see
`examples/simulator.rs`, which runs the same `Theme<Rgb565>` against both a `Rgb565` `SimulatorDisplay` and a
`BinaryColor` one.

### Fonts

Fonts are represented by the `FontKind` enum. There are two variants:

**Bitmap fonts** use the built-in `MonoFont` types from `embedded-graphics`. They work on any target (std or
`no_std`) and require no extra dependencies:

```rust
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_7X13_BOLD};
use iris_ui::{FontKind, Theme, BW_THEME};

let theme = Theme {
font:      FontKind::Bitmap(FONT_7X13),
bold_font: FontKind::Bitmap(FONT_7X13_BOLD),
..BW_THEME
};
```

**TrueType fonts** use [`fontdue`](https://crates.io/crates/fontdue) for scalable, higher-quality text. Enable the
optional `ttf` feature in your `Cargo.toml`:

```toml
iris-ui = { ..., features = ["ttf"] }
```

Then load a font from bytes and pass a reference to a `'static` slot (the example below uses `std`; see the no_std
section for an alternative):

```rust
use std::sync::OnceLock;
use iris_ui::{FontKind, Theme, BW_THEME};

static FONT: OnceLock<fontdue::Font> = OnceLock::new();

let font: & 'static fontdue::Font = FONT.get_or_init(| | {
fontdue::Font::from_bytes(
include_bytes ! ("my_font.ttf"),
fontdue::FontSettings::default(),
).expect("failed to parse font")
});

let theme = Theme {
font:      FontKind::TrueType { font, size: 13.0 },
bold_font: FontKind::TrueType { font, size: 14.0 },
..BW_THEME
};
```

On `no_std + alloc` targets, use `MaybeUninit` in place of `OnceLock`:

```rust
static mut FONT_STORAGE: core::mem::MaybeUninit<fontdue::Font> =
    core::mem::MaybeUninit::uninit();

// called once during init:
let font_ref: & 'static fontdue::Font = unsafe {
FONT_STORAGE.write(
fontdue::Font::from_bytes(
include_bytes!("my_font.ttf"),
fontdue::FontSettings::default (),
).expect("failed to parse font")
);
FONT_STORAGE.assume_init_ref()
};

let theme = Theme {
font:      FontKind::TrueType { font: font_ref, size: 12.0 },
bold_font: FontKind::TrueType { font: font_ref, size: 12.0 },
..BW_THEME
};
```

The simulator example includes a **TTF** button in the font-size toolbar when built with
`cargo run --example simulator --features std,ttf`. It loads a font from a known system path at runtime (Geneva on
macOS, DejaVu Sans on Linux).

## Event Loop

Iris does not provide its own event loop. Instead use whatever loop is provided by the environment you are using. You
will need to receive native input events (touch/mouse down and up, keyboard presses, etc.) and convert them into Iris
events via `pointer_down_at` / `pointer_up_at` (or the `click_at` convenience wrapper, which performs both at once). In
a typical embedded environment, driving a touch controller that only reports "is a finger down right now" requires
tracking the down/up edge yourself, since the toolkit has no built-in debouncing:

```rust
#[main]
fn main() -> ! {
    // set up your board and display
    let mut display: Display<_> = make_display();

    // init your scene
    let mut scene = make_your_scene();

    // create a theme
    let theme = Theme {
        bg: Rgb565::WHITE,
        fg: Rgb565::BLACK,
        selected_bg: Rgb565::WHITE,
        selected_fg: Rgb565::BLACK,
        panel_bg: Rgb565::CSS_LIGHT_GRAY,
        font: FONT_6X10,
        bold_font: FONT_7X13_BOLD,
    };

    // make the drawing context from the display
    let mut ctx = EmbeddedDrawingContext::new(&mut display);

    // init the touch screen
    let touch = Gt911Blocking::default();
    touch.init(i2c_ref).unwrap();

    // event & render loop
    let mut last_touch_point: Option<GPoint> = None;
    loop {

        // handle touch inputs, tracking the down/up edge ourselves
        if let Ok(touch_point) = touch.get_touch(i2c_ref) {
            match touch_point {
                Some(point) => {
                    // flip because the screen is mounted sideways on the t-deck
                    let pt = GPoint::new(320 - point.y as i32, 240 - point.x as i32);
                    if last_touch_point.is_none() {
                        pointer_down_at(&mut scene, &vec![], pt);
                    }
                    last_touch_point = Some(pt);
                }
                None => {
                    if let Some(pt) = last_touch_point.take() {
                        if let Some(result) = pointer_up_at(&mut scene, &vec![], pt) {
                            info!("view returned result {result:?}");
                        }
                    }
                }
            }
        }

        // set up the clip rect
        let delay_start = Instant::now();
        ctx.clip = scene.dirty_rect.clone();

        // draw the scene
        draw_scene(&mut scene, &mut ctx, &theme);

        // wait for 100 msec
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}
```

## Roadmap

### 0.1

- [x] Remove generics for color and font. Just use embedded graphics directly.
- [x] use simulator for interactive tests
- [x] use MockDisplay for automated tests
- [x] support layout using font size. needs padding in the widgets.
- [x] add hbox and vbox layouts
- [x] make children drawn and picked relative to the parent.
- [x] general
    - [x] setup CI on github actions.
- [x] more components
    - [x] add menu view
    - [x] add list view
- [x] drawing
    - [x] redo fill_text api.
        - [x] just text. support bg color?
        - [x] proper alignment. provide center point and draw centered
    - [x] draw line
    - [x] remove clear
    - [x] consolidate Display impls
- [x] layout & rendering
    - [x] calculating dirty rect needs to be converted back to global
    - [x] common view padding
    - [x] new layout algoritm
    - [x] form layout -> grid layout
        - [x] debug lines
        - [x] alignment within grid cells
        - [x] span grid cells
- [x] pick final name

### 0.2

- [x] input improvements
    - [x] cleanup event types and action command signatures.
    - [x] document how to make your own event & draw loop
- [x] text input
    - [x] move cursor within text
    - [x] forward and backward delete
- [ ] focus management
    - [ ] use scroll events to jump between focused elements and perform selection.
    - [ ] spec out how focus management works.
        - [ ] focus groups
- [ ] improved custom view support
    - [ ] view can define the children it uses
        - [ ] let tab panel define its own children using a toggle group
    - [x] let tab panel switch its own tabs instead of using external handle action
- [x] theme accent colors?

### 0.3

- [ ] e-paper support
- [ ] multi-line wrapping text