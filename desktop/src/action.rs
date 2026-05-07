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
        [CopyCell, ConfirmEdit, CancelEdit, StartEdit, SubmitChanges]
    );
}

pub mod app {
    use gpui::actions;
    actions!(app, [Quit]);
}
