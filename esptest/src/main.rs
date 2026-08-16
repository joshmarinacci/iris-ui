#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;
use alloc::vec;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level::{High, Low};
use esp_hal::gpio::{Input, InputConfig, Output, OutputConfig, Pull};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::{Duration, Instant, Rate};
use esp_hal::{main, Blocking};
use iris_ui::button::make_button;
use iris_ui::geom::{Bounds, Point};
use iris_ui::label::make_label;
use iris_ui::scene::Scene;
use iris_ui::scene::{draw_scene, pointer_down_at, pointer_up_at};
use iris_ui::text_input::make_text_input;
use iris_ui::view::{Flex, View, ViewId};
use iris_ui::BW_THEME;
use log::info;

use esp_hal::i2c::master::{BusTimeout, Config as I2CConfig, I2c};
use mipidsi::interface::SpiInterface;
use mipidsi::options::{ColorInversion, ColorOrder, Orientation, Rotation};
use mipidsi::{models::ST7789, Builder};
use static_cell::StaticCell;

use gt911::Gt911Blocking;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::layouts::layout_hbox;
use iris_ui::panel::draw_std_panel;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let mut delay = Delay::new();

    // have to turn on the board and wait 500ms before using the keyboard
    let mut board_power = Output::new(peripherals.GPIO10, High, OutputConfig::default());
    board_power.set_high();
    delay.delay_millis(1000);

    // ==== display setup ====
    // https://github.com/Xinyuan-LilyGO/T-Deck/blob/master/examples/HelloWorld/HelloWorld.ino

    let mut tft_cs = Output::new(peripherals.GPIO12, High, OutputConfig::default());
    tft_cs.set_high();
    let tft_dc = Output::new(peripherals.GPIO11, Low, OutputConfig::default());
    let mut tft_enable = Output::new(peripherals.GPIO42, High, OutputConfig::default());
    tft_enable.set_high();

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)),
    )
        .unwrap()
        .with_sck(peripherals.GPIO40)
        .with_miso(Input::new(
            peripherals.GPIO38,
            InputConfig::default().with_pull(Pull::Up),
        ))
        .with_mosi(peripherals.GPIO41);

    static DISPLAY_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    let buffer = DISPLAY_BUF.init([0u8; 512]);

    info!("setting up the display");
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, tft_cs, spi_delay).unwrap();
    let di = SpiInterface::new(spi_device, tft_dc, buffer);
    info!("building");
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 320)
        .invert_colors(ColorInversion::Inverted)
        .color_order(ColorOrder::Rgb)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut delay)
        .unwrap();

    info!("initialized display");
    // wait for everything to boot up
    // delay.delay_millis(500);
    info!("Display initialized");

    let mut ctx = EmbeddedDrawingContext::new(&mut display);
    let mut scene = make_gui_scene();

    let theme = BW_THEME;

    static I2C: StaticCell<I2c<Blocking>> = StaticCell::new();

    let i2c = I2c::new(
        peripherals.I2C0,
        I2CConfig::default()
            .with_frequency(Rate::from_khz(100))
            .with_timeout(BusTimeout::Disabled),
    )
        .unwrap()
        .with_sda(peripherals.GPIO18)
        .with_scl(peripherals.GPIO8);
    info!("initialized I2C keyboard");
    let i2c_ref = I2C.init(i2c);

    let touch = Gt911Blocking::default();
    touch.init(i2c_ref).unwrap();

    let mut last_touch_point: Option<Point> = None;
    loop {
        // handle touch inputs, with edge-detection so a held finger produces a single
        // PointerDown followed by a single PointerUp on release, rather than repeating.
        if let Ok(touch_point) = touch.get_touch(i2c_ref) {
            match touch_point {
                Some(point) => {
                    // flip because the screen is mounted sideways on the t-deck
                    let pt = Point::new(320 - point.y as i32, 240 - point.x as i32);
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

        let delay_start = Instant::now();
        ctx.clip = scene.dirty_rect.clone();
        draw_scene(&mut scene, &mut ctx, &theme);
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}

fn make_gui_scene() -> Scene {
    let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 320, 240));

    let panel = View {
        name: ViewId::new("panel"),
        bounds: Bounds::new(20, 20, 200, 200),
        draw: Some(draw_std_panel),
        h_flex: Flex::Grow,
        v_flex: Flex::Grow,
        layout: Some(layout_hbox),
        ..Default::default()
    };

    scene.add_view_to_parent(
        make_label("label1", "A Label").position_at(10, 30),
        &panel.name,
    );

    scene.add_view_to_root(make_button(&"button1".into(), "A button").position_at(10, 60));
    scene.add_view_to_root(make_button(&"button2".into(), "A button").position_at(10, 120));

    scene.add_view_to_root(make_button(&"button3".into(), "A button").position_at(10, 200));
    scene.mark_dirty_all();

    scene.add_view_to_parent(
        make_text_input("textinput", "type text here").position_at(10, 90),
        &panel.name,
    );

    scene.add_view_to_root(panel);

    scene
}