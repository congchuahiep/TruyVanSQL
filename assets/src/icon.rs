use gpui::{AnyElement, App, Entity, IntoElement, RenderOnce, SharedString, Window, px};
use gpui_component::{Icon, IconNamed, Sizable};
use gpui_component_macros::icon_named;

icon_named!(AppIcon, "./icons");

impl AppIcon {
    /// Return the icon as a Entity<Icon>
    pub fn view(self, cx: &mut App) -> Entity<Icon> {
        Icon::new(self).view(cx)
    }
}

impl From<AppIcon> for AnyElement {
    fn from(value: AppIcon) -> Self {
        Icon::new(value).into_any_element()
    }
}

impl RenderOnce for AppIcon {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::new(self)
    }
}
