pub mod action;
pub mod connection;
pub mod shared;
pub mod tab_sql_editor;
pub mod tab_table_viewer;
pub mod workspace;

use gpui::*;
use gpui_component::Root;
use workspace::Workspace;

#[tokio::main]
async fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(|cx| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new("ctrl-n", action::toolbar::NewDatabase, Some("app")),
            KeyBinding::new("ctrl-o", action::toolbar::OpenFile, Some("app")),
            KeyBinding::new("ctrl-shift-m", action::toolbar::UseInMemory, Some("app")),
            KeyBinding::new("ctrl-enter", action::query::ExecuteQuery, Some("app")),
            KeyBinding::new("ctrl-c", action::datagrid::CopyCell, Some("data-grid")),
            KeyBinding::new("enter", action::datagrid::StartEdit, Some("data-grid")),
            KeyBinding::new("enter", action::datagrid::ConfirmEdit, Some("cell-editor")),
            KeyBinding::new("escape", action::datagrid::CancelEdit, Some("cell-editor")),
        ]);

        cx.activate(true);

        let window_bounds =
            WindowBounds::Windowed(Bounds::centered(None, size(px(800.0), px(600.0)), cx));

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
                    let view = cx.new(|cx| Workspace::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("Failed to open window");
        })
        .detach();
    });
}
