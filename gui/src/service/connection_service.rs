use gpui::*;

pub struct ConnectionService {
    pub db_url: String,
    pub db_path: String,
}

impl ConnectionService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            db_url: "sqlite::memory:".to_string(),
            db_path: "sqlite::memory:".to_string(),
        }
    }

    pub fn open_file(&mut self, cx: &mut Context<Self>) {
        let this = cx.entity();
        cx.spawn(async move |_, cx| {
            let dialog = rfd::AsyncFileDialog::new()
                .set_title("Mở database")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                .add_filter("All Files", &["*"]);

            if let Some(file) = dialog.pick_file().await {
                let path = file.path().to_string_lossy().to_string();
                let url = format!("sqlite:{path}");
                this.update(cx, |service, cx| {
                    service.db_url = url;
                    service.db_path = path;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn new_database(&mut self, cx: &mut Context<Self>) {
        let this = cx.entity();
        cx.spawn(async move |_, cx| {
            let dialog = rfd::AsyncFileDialog::new()
                .set_title("Tạo database mới")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"]);

            if let Some(file) = dialog.save_file().await {
                let path = file.path().to_string_lossy().to_string();
                let url = format!("sqlite:{path}");
                this.update(cx, |service, cx| {
                    service.db_url = url;
                    service.db_path = path;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn use_in_memory(&mut self, cx: &mut Context<Self>) {
        self.db_url = "sqlite::memory:".to_string();
        self.db_path = "sqlite::memory:".to_string();
        cx.notify();
    }
}