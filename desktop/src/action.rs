pub mod toolbar {
    use gpui::actions;
    actions!(toolbar, [NewDatabase, OpenFile, UseInMemory,]);
}

pub mod sidebar {
    use gpui::actions;
    actions!(sidebar, [RefreshDatabase]);
}

pub mod query {
    use gpui::actions;
    actions!(query, [ExecuteQuery, NewQuery]);
}

pub mod datagrid {
    use gpui::actions;
    actions!(
        grid,
        [CopyCell, ConfirmEdit, CancelEdit, StartEdit, CommitChanges]
    );
}

pub mod app {
    use gpui::actions;
    actions!(app, [Quit]);
}

pub mod theme {
    use gpui::{Action, SharedString};
    use gpui_component::ThemeMode;

    /// Action chuyển sang theme cụ thể theo tên.
    #[derive(Action, Clone, PartialEq)]
    #[action(namespace = theme, no_json)]
    pub struct SwitchTheme(pub SharedString);

    /// Action chuyển đổi Light/Dark mode.
    #[derive(Action, Clone, PartialEq)]
    #[action(namespace = theme, no_json)]
    pub struct SwitchThemeMode(pub ThemeMode);
}
