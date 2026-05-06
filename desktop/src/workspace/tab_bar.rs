use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::tab::{Tab, TabBar as GpuiTabBar};
use gpui_component::{ActiveTheme, IconName, Sizable, h_flex};

use crate::workspace::tab_manager::TabManager;

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

        let mut tab_bar = GpuiTabBar::new("app-tabs")
            .underline()
            .small()
            .w_full()
            .h_8()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background);

        if let Some(index) = active_index {
            tab_bar = tab_bar.selected_index(index);
        }

        tab_bar = tab_bar.on_click(cx.listener(|this, index, _window, cx| {
            this.tab_manager.update(cx, |service, cx| {
                service.select_tab(*index, cx);
            });
        }));

        for (i, any_tab) in tabs.iter().enumerate() {
            let tab_title = any_tab.title(cx);
            let tab_manager_handle = self.tab_manager.clone();

            tab_bar = tab_bar.child(
                Tab::new().label(tab_title).suffix(
                    h_flex().child(
                        Button::new(format!("close-tab-{}", i))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Inbox) // Sẽ thay bằng nút X
                            .on_click(move |_e: &gpui::ClickEvent, _window, cx| {
                                cx.stop_propagation();
                                tab_manager_handle.update(cx, |s, cx| s.close_tab(i, cx));
                            }),
                    ),
                ),
            );
        }

        tab_bar
    }
}
