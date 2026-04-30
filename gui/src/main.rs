mod action;
mod app;
mod panel;
mod query_table_delegate;
mod service;
mod state;
mod util;
mod window_state;

use app::AppView;
use gpui::*;
use gpui_component::Root;

use crate::action::{datagrid, toolbar};

#[tokio::main]
async fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(|cx| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new("ctrl-n", toolbar::NewDatabase, Some("app")),
            KeyBinding::new("ctrl-o", toolbar::OpenFile, Some("app")),
            KeyBinding::new("ctrl-shift-m", toolbar::UseInMemory, Some("app")),
            KeyBinding::new("ctrl-c", datagrid::Copy, Some("datagrid")),
        ]);

        cx.activate(true);

        let window_bounds = match window_state::WindowState::load() {
            Some(state) => state.window_bounds,
            None => WindowBounds::Windowed(Bounds::centered(None, size(px(800.0), px(600.0)), cx)),
        };

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(window_bounds),
                    window_min_size: Some(Size {
                        width: px(640.0),
                        height: px(480.0),
                    }),
                    window_background: WindowBackgroundAppearance::MicaBackdrop,

                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| AppView::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("Failed to open window");
        })
        .detach();
    });
}
