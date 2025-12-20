mod game;
mod patterns;
mod ui;

use gpui::*;
use gpui_component::Root;
use ui::GameOfLife;

fn main() {
    Application::new().run(|app| {
        // Initialize gpui-component
        gpui_component::init(app);

        app.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(100.0), px(100.0)),
                    size: size(px(650.0), px(700.0)),
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Conway's Game of Life".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| GameOfLife::new(cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
    });
}
