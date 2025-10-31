use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::mono_font::MonoFont;
use log::info;
use crate::{DrawEvent, LayoutEvent};
use crate::geom::Size;
use crate::gfx::TextStyle;
use crate::view::{View, ViewId};
use crate::view::Align::Start;
use crate::view::Flex::Shrink;


fn layout_label(e:&mut LayoutEvent) {
    if let Some(view) = e.scene.get_view_mut(e.target) {
        let res = line_wrap(e.theme.font, &view.title, e.space);
        view.bounds.size = res.0;
    }
}

fn line_wrap(font: MonoFont, text: &String, avail:Size) -> (Size,Vec<String>) {
    let fsize = &font.character_size;

    let mut rows:Vec<String> = vec![];
    let mut size = Size::new(0, fsize.height as i32);
    let mut index:usize = 0;
    let max_chars:usize = (avail.w / fsize.width as i32) as usize;
    let mut longest = 0;
    loop {
        let next:usize = if index + max_chars < text.len() {
            index + max_chars
        } else {
            text.len()
        };
        let len = next - index;
        let width = len as u32 * fsize.width;
        rows.push(text[index..next].to_string());
        longest = longest.max(width);

        index = next;
        if index + 1 > text.len() {
            break;
        }
    }
    size.w = longest as i32;
    let vpad = (fsize.height as i32)* 2;
    size.h = rows.len() as i32 * fsize.height as i32 + vpad;

    (size,rows)
}

fn draw_label(e:&mut DrawEvent) {
    let style = TextStyle::new(&e.theme.font, &e.theme.accented.fill).with_halign(Start);
    let res = line_wrap(e.theme.font, &e.view.title, e.view.bounds.size);
    let line_height = e.theme.font.character_size.height as i32;
    let mut pos = e.view.bounds.position.clone();
    pos.y += line_height/2;
    for row in res.1 {
        pos.y += line_height;
        e.ctx.text(&row, &pos, &style);
    }
}

pub fn make_multi_label(name: &ViewId, title: &str) -> View {
    View {
        name: name.clone(),
        title: title.into(),
        h_flex: Shrink,
        v_flex: Shrink,
        layout: Some(layout_label),
        draw: Some(draw_label),
        ..Default::default()
    }
}

mod tests {
    use log::info;
    use crate::geom::{Bounds, Size};
    use crate::multiline_label::make_multi_label;
    use crate::scene::{layout_scene, layout_view, Scene};
    use crate::test::MockDrawingContext;
    use crate::view::{Flex, ViewId};
    use test_log::test;

    #[test]
    fn short_label() {
        info!("starting multiline-label test");
        let theme = MockDrawingContext::make_mock_theme();
        let mut scene = Scene::new_with_bounds(Bounds::new(0, 0, 30, 240));

        // layout foo bar baz with plenty of space
        {
            let name = ViewId::new("label1");
            let label = make_multi_label(&name, "foo bar baz")
                .with_h_flex(Flex::Fixed).with_bounds(Bounds::new(0,0,200,200));
            scene.add_view_to_root(label);

            layout_view(&mut scene, &theme, &name, Size::new(200, 240));
            let label = scene.get_view(&name).unwrap();
            assert_eq!(label.bounds, Bounds::new(0,0,66,30));
        }

        // layout foo bar baz with little space so it must wrap
        {
            let name = ViewId::new("label2");
            let label = make_multi_label(&name, "foo bar baz")
                .with_h_flex(Flex::Fixed).with_bounds(Bounds::new(0,0,200,200));
            scene.add_view_to_root(label);

            layout_view(&mut scene, &theme, &name, Size::new(30, 240));
            let label = scene.get_view(&name).unwrap();
            assert_eq!(label.bounds, Bounds::new(0,0,30,50));
        }

        // test that vbox is wrapping label properly to the width
        // let vbox = make_panel(&ViewId::new("vbox")).with_h_flex(Flex::Grow);
        // test that hbox is centering vertically when wrapped with label fixed width
        // test that hbox is centering vertically when wrapped with label grow width
    }
}
