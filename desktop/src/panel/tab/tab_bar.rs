use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::Sizable;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::h_flex;

use crate::component::tab::Tab;
use assets::AppIcon;

use crate::panel::tab::TabManager;

pub struct TabBar {
    tab_manager: Entity<TabManager>,
}

impl TabBar {
    pub fn new(tab_manager: Entity<TabManager>, cx: &mut Context<Self>) -> Self {
        cx.observe(&tab_manager, |_, _, cx| cx.notify()).detach();
        Self { tab_manager }
    }
}

impl Render for TabBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let manager = self.tab_manager.read(cx);
        let active_index = manager.active_index();
        let tabs = manager.tabs();
        let tab_manager_handle = self.tab_manager.clone();

        div()
            .id("tab-bar")
            .flex()
            .items_stretch()
            .h_8()
            .w_full()
            .bg(cx.theme().tab_bar)
            .child(
                div()
                    .id("border-b")
                    .absolute()
                    .left_0()
                    .bottom_0()
                    .size_full()
                    .border_b_1()
                    .border_color(cx.theme().border),
            )
            .border_color(cx.theme().border)
            .when(tabs.len() == 0, |this| this.hidden())
            .child(
                h_flex()
                    .id("tabs-scroll")
                    .flex_1()
                    .overflow_x_scroll()
                    .items_stretch()
                    .children(tabs.iter().enumerate().map(|(i, tab)| {
                        let info = tab.info(cx);
                        let tab_title = info.title;
                        let is_dirty = info.is_dirty;
                        let icon = info.icon;
                        let is_selected = active_index == Some(i);

                        let tab_manager_for_left_click = tab_manager_handle.clone();
                        let tab_manager_for_middle_click = tab_manager_handle.clone();
                        let tab_manager_for_close = tab_manager_handle.clone();

                        Tab::new()
                            .ix(i)
                            .non_border_l(i == 0)
                            .selected(is_selected)
                            .label(tab_title)
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                tab_manager_for_left_click
                                    .update(cx, |service, cx| service.select_tab(i, cx))
                            })
                            .on_mouse_down(MouseButton::Middle, move |_event, _window, cx| {
                                tab_manager_for_middle_click
                                    .update(cx, |service, cx| service.close_tab(i, cx))
                            })
                            .icon(icon)
                            .dirtied(is_dirty)
                            .suffix(
                                h_flex().child(
                                    Button::new(format!("close-tab-{}", i))
                                        .ghost()
                                        .xsmall()
                                        .mr_1()
                                        .cursor_pointer()
                                        .icon(AppIcon::X)
                                        .on_click(move |_e, _window, cx| {
                                            cx.stop_propagation();
                                            tab_manager_for_close
                                                .update(cx, |service, cx| service.close_tab(i, cx));
                                        }),
                                ),
                            )
                            .into_any_element()
                    })),
            )
    }
}