#![allow(dead_code)]

use assets::AppIcon;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    percentage, px,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::menu::{ContextMenuExt, PopupMenu};
use gpui_component::sidebar::SidebarItem;
use gpui_component::{
    ActiveTheme as _, Collapsible, Icon, Sizable as _, StyledExt, h_flex, v_flex,
};
use std::rc::Rc;

#[derive(Clone)]
pub struct SidebarMenuItem {
    icon: Option<Icon>,
    label: SharedString,
    handler: Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>,
    on_double_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    expand_on_double_click: bool,
    loading: bool,
    active: bool,
    default_open: bool,
    collapsed: bool,
    children: Vec<Self>,
    suffix: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
    disabled: bool,
    context_menu: Option<Rc<dyn Fn(PopupMenu, &mut Window, &mut App) -> PopupMenu + 'static>>,
}

impl SidebarMenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            label: label.into(),
            handler: Rc::new(|_, _, _| {}),
            on_double_click: None,
            expand_on_double_click: false,
            loading: false,
            active: false,
            collapsed: false,
            default_open: false,
            children: Vec::new(),
            suffix: None,
            disabled: false,
            context_menu: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handler = Rc::new(handler);
        self
    }

    pub fn on_double_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_double_click = Some(Rc::new(handler));
        self
    }

    pub fn expand_on_double_click(mut self, enable: bool) -> Self {
        self.expand_on_double_click = enable;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<Self>>) -> Self {
        self.children = children.into_iter().map(Into::into).collect();
        self
    }

    pub fn suffix<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement,
    {
        self.suffix = Some(Rc::new(move |window, cx| {
            builder(window, cx).into_any_element()
        }));
        self
    }

    pub fn disable(mut self, disable: bool) -> Self {
        self.disabled = disable;
        self
    }

    fn is_submenu(&self) -> bool {
        !self.children.is_empty()
    }

    fn needs_caret(&self) -> bool {
        self.is_submenu() || self.expand_on_double_click || self.loading
    }

    pub fn context_menu(
        mut self,
        f: impl Fn(PopupMenu, &mut Window, &mut App) -> PopupMenu + 'static,
    ) -> Self {
        self.context_menu = Some(Rc::new(f));
        self
    }
}

impl Collapsible for SidebarMenuItem {
    fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

impl SidebarItem for SidebarMenuItem {
    fn render(
        self,
        id: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let has_double_click = self.on_double_click.is_some() || self.expand_on_double_click;
        let expand_on_double_click = self.expand_on_double_click;
        let is_loading = self.loading;
        let default_open = self.default_open;
        let id = id.into();
        let handler = self.handler.clone();
        let double_click_handler = self.on_double_click.clone();
        let is_collapsed = self.collapsed;
        let is_active = self.active;
        let is_hoverable = !is_active && !self.disabled;
        let is_disabled = self.disabled;
        let needs_caret = self.needs_caret();

        let open_state = if needs_caret {
            Some(window.use_keyed_state(id.clone(), cx, |_, _| default_open))
        } else {
            None
        };

        let last_click_state = if has_double_click {
            Some(window.use_keyed_state(
                ElementId::Name(format!("{}-dblclick", id).into()),
                cx,
                |_, _| 0u64,
            ))
        } else {
            None
        };

        let is_open = open_state
            .as_ref()
            .map_or(false, |s| !is_collapsed && *s.read(cx));

        div()
            .id(id.clone())
            .w_full()
            .child(
                h_flex()
                    .id("item")
                    .size_full()
                    .overflow_x_hidden()
                    .flex_shrink_0()
                    .p_1()
                    .cursor_pointer()
                    .gap_x(px(5.))
                    .rounded(cx.theme().radius)
                    .text_sm()
                    .when(is_hoverable, |this| {
                        this.hover(|this| {
                            this.bg(cx.theme().sidebar_accent.opacity(0.8))
                                .text_color(cx.theme().sidebar_accent_foreground)
                        })
                    })
                    .when(is_active, |this| {
                        this.font_medium()
                            .bg(cx.theme().sidebar_accent)
                            .text_color(cx.theme().sidebar_accent_foreground)
                    })
                    .when(is_collapsed, |this| {
                        this.justify_center().when(is_active, |this| {
                            this.bg(cx.theme().sidebar_accent)
                                .text_color(cx.theme().sidebar_accent_foreground)
                        })
                    })
                    .when(!is_collapsed, |this| {
                        this.h_7()
                            .when(is_loading, |this| {
                                this.child(
                                    Icon::new(AppIcon::Loader).size_4(),
                                )
                            })
                            .when_some(open_state.clone(), |this, open_state| {
                                let is_open_for_caret = is_open;
                                this.child(
                                    Button::new("caret")
                                        .xsmall()
                                        .ghost()
                                        .icon(
                                            Icon::new(AppIcon::ChevronRight)
                                                .size_4()
                                                .when(is_open_for_caret, |this| {
                                                    this.rotate(percentage(90. / 360.))
                                                }),
                                        )
                                        .on_click({
                                            move |_, _, cx| {
                                                cx.stop_propagation();
                                                open_state.update(cx, |is_open, cx| {
                                                    *is_open = !*is_open;
                                                    cx.notify();
                                                })
                                            }
                                        }),
                                )
                            })
                            .child(
                                h_flex()
                                    .flex_1()
                                    .gap_x_1()
                                    .justify_between()
                                    .overflow_x_hidden()
                                    .child(
                                        h_flex()
                                            .flex_1()
                                            .overflow_x_hidden()
                                            .gap_x_1()
                                            .when_some(self.icon.clone(), |this, icon| {
                                                this.child(icon)
                                            })
                                            .child(self.label.clone()),
                                    )
                                    .when_some(self.suffix.clone(), |this, suffix| {
                                        this.child(suffix(window, cx).into_any_element())
                                    }),
                            )
                    })
                    .when(is_disabled, |this| {
                        this.text_color(cx.theme().muted_foreground)
                    })
                    .when(!is_disabled, |this| {
                        this.on_click({
                            let open_state = open_state.clone();
                            let last_click_state = last_click_state.clone();
                            move |ev, window, cx| {
                                if has_double_click {
                                    if let Some(ref lc) = last_click_state {
                                        let now = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis()
                                            as u64;
                                        let last = *lc.read(cx);
                                        if now - last < 300 {
                                            lc.update(cx, |ts, _| *ts = 0);
                                            if expand_on_double_click {
                                                if let Some(ref s) = open_state {
                                                    s.update(cx, |is_open, cx| {
                                                        *is_open = !*is_open;
                                                        cx.notify();
                                                    });
                                                }
                                            }
                                            if let Some(ref dbl) = double_click_handler {
                                                dbl(ev, window, cx);
                                            }
                                            return;
                                        }
                                        lc.update(cx, |ts, _| *ts = now);
                                    }
                                    handler(ev, window, cx);
                                } else {
                                    if let Some(ref s) = open_state {
                                        s.update(cx, |is_open, cx| {
                                            *is_open = !*is_open;
                                            cx.notify();
                                        });
                                    }
                                    handler(ev, window, cx);
                                }
                            }
                        })
                    })
                    .map(|this| {
                        if let Some(context_menu) = self.context_menu {
                            this.context_menu(move |menu, window, cx| {
                                context_menu(menu, window, cx)
                            })
                            .into_any_element()
                        } else {
                            this.into_any_element()
                        }
                    }),
            )
            .when(is_open, |this| {
                this.child(
                    v_flex()
                        .id("submenu")
                        .border_l_1()
                        .border_color(cx.theme().sidebar_border)
                        .gap_1()
                        .ml(px(14.))
                        .pl_2p5()
                        .py_0p5()
                        .children(self.children.into_iter().enumerate().map(|(ix, item)| {
                            let child_id = format!("{}-{}", id, ix);
                            item.render(child_id, window, cx).into_any_element()
                        })),
                )
            })
    }
}

impl From<AppIcon> for SidebarMenuItem {
    fn from(icon: AppIcon) -> Self {
        Self::new("").icon(icon)
    }
}

impl From<&'static str> for SidebarMenuItem {
    fn from(label: &'static str) -> Self {
        Self::new(label)
    }
}

impl From<String> for SidebarMenuItem {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

impl From<SharedString> for SidebarMenuItem {
    fn from(label: SharedString) -> Self {
        Self::new(label)
    }
}