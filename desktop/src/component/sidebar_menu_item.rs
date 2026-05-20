#![allow(dead_code)]

use assets::AppIcon;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    percentage,
};
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::menu::{ContextMenuExt, PopupMenu};
use gpui_component::sidebar::SidebarItem;
use gpui_component::spinner::Spinner;
use gpui_component::{
    ActiveTheme as _, Collapsible, Icon, Sizable as _, StyledExt, h_flex, v_flex,
};
use std::rc::Rc;

#[derive(Clone)]
pub struct SidebarMenuItem {
    icon: Option<Icon>,
    label: SharedString,
    handler: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    on_double_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    on_expand: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    double_click_to_expand: bool,
    indented: bool,
    loading: bool,
    active: bool,
    default_open: bool,
    children: Vec<Self>,
    suffix: Option<Rc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
    disabled: bool,
    context_menu: Option<Rc<dyn Fn(PopupMenu, &mut Window, &mut App) -> PopupMenu + 'static>>,
}

impl Collapsible for SidebarMenuItem {
    fn is_collapsed(&self) -> bool {
        false
    }

    fn collapsed(self, _collapsed: bool) -> Self {
        self
    }
}

impl SidebarMenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            label: label.into(),
            handler: None,
            on_double_click: None,
            on_expand: None,
            double_click_to_expand: true,
            indented: false,
            loading: false,
            active: false,
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
        self.handler = Some(Rc::new(handler));
        self
    }

    pub fn on_double_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_double_click = Some(Rc::new(handler));
        self
    }

    pub fn on_expand(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Rc::new(handler));
        self
    }

    pub fn double_click_to_expand(mut self, enable: bool) -> Self {
        self.double_click_to_expand = enable;
        self
    }

    pub fn indented(mut self, indented: bool) -> Self {
        self.indented = indented;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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
        self.is_submenu() || self.on_expand.is_some() || self.double_click_to_expand || self.loading
    }

    pub fn context_menu(
        mut self,
        f: impl Fn(PopupMenu, &mut Window, &mut App) -> PopupMenu + 'static,
    ) -> Self {
        self.context_menu = Some(Rc::new(f));
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
        let has_double_click = self.on_double_click.is_some() || self.double_click_to_expand;
        let double_click_to_expand = self.double_click_to_expand;
        let loading = self.loading;
        let default_open = self.default_open;
        let id = id.into();
        let click_handler = self.handler.clone();
        let double_click_handler = self.on_double_click.clone();
        let on_expand = self.on_expand.clone();
        let is_active = self.active;
        let is_hoverable = !is_active && !self.disabled;
        let is_disabled = self.disabled;
        let needs_caret = self.needs_caret();
        let indented = self.indented;

        let open_state = if needs_caret {
            Some(window.use_keyed_state(id.clone(), cx, |_, _| default_open))
        } else {
            None
        };

        let open_state_for_caret = open_state.clone();
        let open_state_for_click = open_state.clone();

        let last_click_state = if has_double_click {
            Some(window.use_keyed_state(
                ElementId::Name(format!("{}-dblclick", id).into()),
                cx,
                |_, _| 0u64,
            ))
        } else {
            None
        };

        let is_open = open_state.as_ref().map_or(false, |s| *s.read(cx));

        div()
            .id(id.clone())
            .w_full()
            .child(
                h_flex()
                    .id("item")
                    .size_full()
                    .overflow_x_hidden()
                    .flex_shrink_0()
                    .cursor_pointer()
                    .p_1()
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
                    .h_7()
                    .when(needs_caret, |this| {
                        this.when_else(
                            loading,
                            |this| this.pl_2().child(Spinner::new()),
                            |this| {
                                this.child(
                                    Button::new("caret")
                                        .xsmall()
                                        .ghost()
                                        .cursor_pointer()
                                        .icon(
                                            Icon::new(AppIcon::ChevronRight)
                                                .size_4()
                                                .when(is_open, |this| {
                                                    this.rotate(percentage(90. / 360.))
                                                }),
                                        )
                                        .on_click({
                                            let on_expand = on_expand.clone();
                                            move |_, window, cx| {
                                                cx.stop_propagation();
                                                if let Some(ref entity) = open_state_for_caret {
                                                    let mut was_expanding = false;
                                                    entity.update(cx, |is_open, _cx| {
                                                        was_expanding = !*is_open;
                                                        *is_open = !*is_open;
                                                    });
                                                    if was_expanding {
                                                        if let Some(ref handler) = on_expand {
                                                            handler(window, cx);
                                                        }
                                                    }
                                                }
                                            }
                                        }),
                                )
                            },
                        )
                    })
                    .when(!needs_caret && indented, |this| this.pl_6())
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
                                    .when_some(self.icon.clone(), |this, icon| this.child(icon))
                                    .child(self.label.clone()),
                            )
                            .when_some(self.suffix.clone(), |this, suffix| {
                                this.child(suffix(window, cx).into_any_element())
                            }),
                    )
                    .when(is_disabled, |this| {
                        this.text_color(cx.theme().muted_foreground)
                    })
                    .when(!is_disabled, |this| {
                        this.on_click({
                            let open_state = open_state_for_click.clone();
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

                                            if double_click_to_expand {
                                                if let Some(ref entity) = open_state {
                                                    let mut was_expanding = false;
                                                    entity.update(cx, |is_open, _cx| {
                                                        was_expanding = !*is_open;
                                                        *is_open = !*is_open;
                                                    });
                                                    if was_expanding {
                                                        if let Some(ref handler) = on_expand {
                                                            handler(window, cx);
                                                        }
                                                    }
                                                }
                                            }

                                            if let Some(ref handler) = double_click_handler {
                                                handler(ev, window, cx);
                                            }
                                            return;
                                        }
                                        lc.update(cx, |ts, _| *ts = now);
                                    }

                                    if let Some(ref handler) = click_handler {
                                        handler(ev, window, cx);
                                    }
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
                        .ml(gpui::px(13.))
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
