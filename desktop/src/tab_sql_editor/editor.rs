use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub struct QueryEditor {
    sql_input: Entity<InputState>,
}

impl QueryEditor {
    pub fn new(sql_input: Entity<InputState>) -> Self {
        Self { sql_input }
    }
}

impl Render for QueryEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                h_flex()
                    .px_3()
                    .py_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        Label::new("SQL Editor")
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .p_1()
                    .bg(cx.theme().background)
                    .child(Input::new(&self.sql_input).h_full()),
            )
    }
}
