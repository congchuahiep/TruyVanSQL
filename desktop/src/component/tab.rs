use std::rc::Rc;

use assets::AppIcon;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, ClickEvent, Div, Edges, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
    relative,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::spinner::Spinner;
use gpui_component::{ActiveTheme, Icon, IconName, Selectable, Sizable};

#[allow(dead_code)]
struct TabStyle {
    borders: Edges<Pixels>,
    border_color: Hsla,
    bg: Hsla,
    fg: Hsla,
}

impl Default for TabStyle {
    fn default() -> Self {
        TabStyle {
            borders: Edges::all(px(0.)),
            border_color: gpui::transparent_white(),
            bg: gpui::transparent_white(),
            fg: gpui::transparent_white(),
        }
    }
}

fn style_normal(cx: &App) -> TabStyle {
    TabStyle {
        fg: cx.theme().tab_foreground,
        bg: cx.theme().transparent,
        borders: Edges {
            left: px(1.),
            right: px(1.),
            ..Default::default()
        },
        border_color: cx.theme().transparent,
    }
}

fn style_hovered(_selected: bool, cx: &App) -> TabStyle {
    TabStyle {
        fg: cx.theme().tab_active_foreground,
        bg: cx.theme().transparent,
        borders: Edges {
            left: px(1.),
            right: px(1.),
            ..Default::default()
        },
        border_color: cx.theme().transparent,
    }
}

fn style_selected(cx: &App) -> TabStyle {
    TabStyle {
        fg: cx.theme().tab_active_foreground,
        bg: cx.theme().tab_active,
        borders: Edges {
            left: px(1.),
            right: px(1.),
            ..Default::default()
        },
        border_color: cx.theme().border,
    }
}

fn style_disabled(selected: bool, cx: &App) -> TabStyle {
    TabStyle {
        fg: cx.theme().muted_foreground,
        bg: cx.theme().transparent,
        borders: Edges {
            left: px(1.),
            right: px(1.),
            ..Default::default()
        },
        border_color: if selected {
            cx.theme().border
        } else {
            cx.theme().transparent
        },
    }
}

#[derive(IntoElement)]
pub struct Tab {
    ix: usize,
    base: Div,
    label: Option<SharedString>,
    icon: Option<Icon>,
    prefix: Option<AnyElement>,
    non_border_l: Option<bool>,
    suffix: Option<AnyElement>,
    children: Vec<AnyElement>,
    disabled: bool,
    selected: bool,
    dirtied: bool,
    loading: bool,
    close_button: bool,
    indicator_active: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_close: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl From<&'static str> for Tab {
    fn from(label: &'static str) -> Self {
        Self::new().label(label)
    }
}

impl From<String> for Tab {
    fn from(label: String) -> Self {
        Self::new().label(label)
    }
}

impl From<SharedString> for Tab {
    fn from(label: SharedString) -> Self {
        Self::new().label(label)
    }
}

impl From<Icon> for Tab {
    fn from(icon: Icon) -> Self {
        Self::default().icon(icon)
    }
}

impl From<IconName> for Tab {
    fn from(icon_name: IconName) -> Self {
        Self::default().icon(Icon::new(icon_name))
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            ix: 0,
            base: div(),
            label: None,
            icon: None,
            prefix: None,
            non_border_l: None,
            suffix: None,
            children: Vec::new(),
            disabled: false,
            selected: false,
            dirtied: false,
            loading: false,
            close_button: false,
            indicator_active: false,
            on_click: None,
            on_close: None,
        }
    }
}

impl Tab {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn dirtied(mut self, dirtied: bool) -> Self {
        self.dirtied = dirtied;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn close_button(mut self, close_button: bool) -> Self {
        self.close_button = close_button;
        self
    }

    pub fn on_close(
        mut self,
        on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(on_close));
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    pub(crate) fn ix(mut self, ix: usize) -> Self {
        self.ix = ix;
        self
    }

    pub(crate) fn non_border_l(mut self, non_border_l: bool) -> Self {
        self.non_border_l = Some(non_border_l);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl ParentElement for Tab {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Selectable for Tab {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl InteractiveElement for Tab {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Tab {}

impl Styled for Tab {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Tab {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tab_style = if self.disabled {
            style_disabled(self.selected, cx)
        } else if self.selected {
            style_selected(cx)
        } else {
            style_normal(cx)
        };

        let hover_style = if self.disabled {
            style_disabled(self.selected, cx)
        } else {
            style_hovered(self.selected, cx)
        };

        let non_border_l = self.non_border_l.unwrap_or_default();
        let (borders_left, h_borders_left, border_color, h_border_color) = if non_border_l {
            (
                px(0.),
                px(0.),
                tab_style.border_color,
                hover_style.border_color,
            )
        } else {
            (
                tab_style.borders.left,
                hover_style.borders.left,
                tab_style.border_color,
                hover_style.border_color,
            )
        };

        let icon_element = if self.loading {
            Some(Spinner::new().with_size(px(14.)).into_any_element())
        } else {
            self.icon
                .map(|icon| icon.with_size(px(14.)).into_any_element())
        };

        self.base
            .id(self.ix)
            .relative()
            .flex()
            .gap_1()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .h_8()
            .pl_5()
            .when_else(self.close_button, |this| this.pr_7(), |this| this.pr_5())
            .line_height(relative(1.))
            .whitespace_nowrap()
            .overflow_hidden()
            .text_color(tab_style.fg)
            .text_sm()
            .bg(tab_style.bg)
            .border_l(borders_left)
            .border_r(tab_style.borders.right)
            .border_color(border_color)
            .rounded_t_lg()
            .when(!self.selected && !self.disabled, |this| {
                this.hover(|this| {
                    this.text_color(hover_style.fg)
                        .bg(hover_style.bg)
                        .border_l(h_borders_left)
                        .border_r(hover_style.borders.right)
                        .border_color(h_border_color)
                })
            })
            .when_some(self.prefix, |this, prefix| this.child(prefix))
            .when(self.dirtied, |this| {
                this.child(
                    div()
                        .absolute()
                        .left_2()
                        .size_1p5()
                        .rounded_full()
                        .bg(cx.theme().blue),
                )
            })
            .when_some(icon_element, |this, icon| this.child(icon))
            .when_some(self.label, |this, label| {
                this.child(div().child(label).pb_0p5())
            })
            .when_some(self.suffix, |this, suffix| this.child(suffix))
            .when(self.close_button, |this| {
                this.child(
                    Button::new(format!("close-tab-{}", self.ix))
                        .absolute()
                        .right_1()
                        .ghost()
                        .xsmall()
                        .cursor_pointer()
                        .icon(AppIcon::X)
                        .when_some(self.on_close.clone(), |this, on_close| {
                            this.on_click(move |event, window, cx| on_close(event, window, cx))
                        }),
                )
            })
            .when(!self.disabled, |this| {
                this.when_some(self.on_click.clone(), |this, on_click| {
                    this.on_click(move |event, window, cx| on_click(event, window, cx))
                })
            })
    }
}
