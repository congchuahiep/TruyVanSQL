use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, ClickEvent, Div, Edges, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
    relative,
};
use gpui_component::{ActiveTheme, Icon, IconName, Selectable, Sizable, Size, StyledExt, h_flex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabVariant {
    Tab,
}

impl TabVariant {
    fn height(self, size: Size) -> Pixels {
        match size {
            Size::XSmall => px(20.),
            Size::Small => px(24.),
            Size::Large => px(36.),
            _ => px(32.),
        }
    }

    pub(super) fn inner_height(self, size: Size) -> Pixels {
        match size {
            Size::XSmall => px(18.),
            Size::Small => px(22.),
            Size::Large => px(36.),
            _ => px(30.),
        }
    }

    fn inner_paddings(self, size: Size) -> Edges<Pixels> {
        let padding_x = match size {
            Size::XSmall => px(8.),
            Size::Small => px(10.),
            Size::Large => px(16.),
            _ => px(12.),
        };
        Edges {
            left: padding_x,
            right: padding_x,
            ..Default::default()
        }
    }
}

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
    size: Size,
    disabled: bool,
    selected: bool,
    dirtied: bool,
    indicator_active: bool,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
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
            size: Size::default(),
            disabled: false,
            selected: false,
            dirtied: false,
            indicator_active: false,
            on_click: None,
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

impl Sizable for Tab {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
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

        let inner_height = TabVariant::Tab.inner_height(self.size);
        let inner_paddings = TabVariant::Tab.inner_paddings(self.size);
        let height = TabVariant::Tab.height(self.size);

        self.base
            .id(self.ix)
            .flex()
            .flex_wrap()
            .gap_1()
            .items_center()
            .flex_shrink_0()
            .h(height)
            .overflow_hidden()
            .text_color(tab_style.fg)
            .map(|this| match self.size {
                Size::XSmall => this.text_xs(),
                Size::Large => this.text_base(),
                _ => this.text_sm(),
            })
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
            .child(
                h_flex()
                    .flex_1()
                    .ml_2()
                    .relative()
                    .h(inner_height)
                    .line_height(relative(1.))
                    .whitespace_nowrap()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .gap_1()
                    .flex_shrink_0()
                    .paddings(inner_paddings)
                    .when(self.dirtied, |this| {
                        this.child(
                            div()
                                .size_1p5()
                                .rounded_full()
                                .bg(cx.theme().blue)
                                .absolute()
                                .left_0(),
                        )
                    })
                    .when_some(self.icon, |this, icon| this.child(icon).mb_neg_1())
                    .when_some(self.label, |this, label| this.child(label)),
            )
            .when_some(self.suffix, |this, suffix| this.child(suffix))
            .when(!self.disabled, |this| {
                this.when_some(self.on_click.clone(), |this, on_click| {
                    this.on_click(move |event, window, cx| on_click(event, window, cx))
                })
            })
    }
}
