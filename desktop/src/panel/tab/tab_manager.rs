use crate::panel::tab::tab_item::{TabInfo, TabItem};
use gpui::*;
use std::sync::Arc;

/// Wrapper che giấu kiểu dữ liệu thật của Tab.
/// Lưu trữ `AnyView` và một closure để lấy title.
#[derive(Clone)]
pub struct AnyTab {
    view: AnyView,
    info_fn: Arc<dyn Fn(&App) -> TabInfo>,
}

impl AnyTab {
    pub fn new<T: TabItem>(view: Entity<T>) -> Self {
        Self {
            view: view.clone().into(),
            info_fn: Arc::new(move |cx| view.read(cx).tab_info(cx)),
        }
    }

    pub fn info(&self, cx: &App) -> TabInfo {
        (self.info_fn)(cx)
    }

    pub fn view(&self) -> AnyView {
        self.view.clone()
    }
}

/// Dịch vụ quản lý các Tab đa hình.
pub struct TabManager {
    tabs: Vec<AnyTab>,
    active_index: Option<usize>,
}

impl TabManager {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            tabs: Vec::new(),
            active_index: None,
        }
    }

    /// Mở một tab bất kỳ miễn là nó implement TabItem.
    pub fn open_tab<T: TabItem>(&mut self, tab_view: Entity<T>, cx: &mut Context<Self>) {
        self.tabs.push(AnyTab::new(tab_view));
        self.active_index = Some(self.tabs.len() - 1);
        cx.notify();
    }

    pub fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if let Some(active) = self.active_index {
                if self.tabs.is_empty() {
                    self.active_index = None;
                } else if active >= self.tabs.len() {
                    self.active_index = Some(self.tabs.len() - 1);
                }
            }
            cx.notify();
        }
    }

    pub fn select_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.active_index = Some(index);
            cx.notify();
        }
    }

    pub fn active_tab(&self) -> Option<AnyTab> {
        self.active_index.and_then(|i| self.tabs.get(i).cloned())
    }

    pub fn tabs(&self) -> &[AnyTab] {
        &self.tabs
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active_index
    }
}