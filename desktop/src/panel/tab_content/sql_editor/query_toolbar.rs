use crate::panel::tab_content::sql_editor::query_session::QuerySession;
use assets::AppIcon;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, Disableable, h_flex};

pub struct QueryToolbar {
    session: Entity<QuerySession>,
}

impl QueryToolbar {
    pub fn new(session: Entity<QuerySession>, cx: &mut Context<Self>) -> Self {
        cx.observe(&session, |_, _, cx| cx.notify()).detach();
        Self { session }
    }

    fn on_execute(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.session.update(cx, |s, cx| s.execute(cx));
    }
}

impl Render for QueryToolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_executing = self.session.read(cx).is_executing;
        let status = if is_executing {
            "Đang thực thi..."
        } else {
            "Sẵn sàng"
        };

        h_flex()
            .px_3()
            .py_2()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                Button::new("btn-execute")
                    .icon(AppIcon::Play)
                    .label("Execute")
                    .primary()
                    .disabled(is_executing)
                    .on_click(cx.listener(Self::on_execute)),
            )
            .child(div().flex_1())
            .child(
                Label::new(status)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
    }
}