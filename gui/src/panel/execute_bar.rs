use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, Disableable, IconName, h_flex};

use crate::service::query_service::QueryService;

pub struct ExecuteBar {
    query: Entity<QueryService>,
}

impl ExecuteBar {
    pub fn new(query: Entity<QueryService>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.observe(&query, |_, _, cx| cx.notify()).detach();
        Self { query }
    }

    fn on_execute(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.query.update(cx, |q, cx| q.execute(cx));
    }
}

impl Render for ExecuteBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_executing = self.query.read(cx).is_executing;
        let status = if is_executing {
            "Đang thực thi..."
        } else {
            "Sẵn sàng"
        };

        h_flex()
            .w_full()
            .px_3()
            .py_2()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                Button::new("btn-execute")
                    .icon(IconName::Play)
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
