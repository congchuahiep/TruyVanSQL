use std::collections::HashMap;

use engine::ConnectionCategory;
use engine::{DatabaseConfig, DatabaseKind, SqlClient};
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::form::{field, h_form};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::{ActiveTheme, Disableable, IndexPath, Sizable, h_flex, v_flex};

use crate::connection::ConnectionStore;
use crate::panel::Titlebar;

/// Danh sách các loại database hỗ trợ trong dropdown.
const DATABASE_KINDS: [&str; 2] = ["SQLite", "PostgreSQL"];

pub struct ConnectionWindow {
    is_testing: bool,
    scroll_handle: ScrollHandle,
    title_bar: Entity<Titlebar>,

    selected_kind: DatabaseKind,

    kind_select: Entity<SelectState<Vec<SharedString>>>,
    name_input: Entity<InputState>,

    host_input: Entity<InputState>,
    port_input: Entity<InputState>,
    user_input: Entity<InputState>,
    password_input: Entity<InputState>,
    database_input: Entity<InputState>,

    selected_path: Option<String>,

    form_errors: HashMap<&'static str, SharedString>,

    connection_store: Entity<ConnectionStore>,
}

impl ConnectionWindow {
    pub fn new(
        connection_store: Entity<ConnectionStore>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title_bar = cx.new(|cx| Titlebar::dialog("Kết nối Database", cx));

        let selected_kind = DatabaseKind::Postgres;
        let kind_select = cx.new(|cx| {
            SelectState::new(
                DATABASE_KINDS.iter().map(|s| (*s).into()).collect(),
                Some(IndexPath::default().row(1)),
                window,
                cx,
            )
            .searchable(true)
        });
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tên kết nối")
                .default_value("")
        });

        let host_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("localhost")
                .default_value("localhost")
        });
        let port_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("5432")
                .default_value("5432")
        });
        let user_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("postgres")
                .default_value("postgres")
        });
        let password_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu")
                .masked(true)
        });
        let database_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("postgres")
                .default_value("postgres")
        });

        let mut this = Self {
            is_testing: false,
            scroll_handle: ScrollHandle::new(),
            title_bar,
            selected_kind,
            kind_select,
            name_input,
            host_input,
            port_input,
            user_input,
            password_input,
            database_input,
            selected_path: None,
            form_errors: HashMap::new(),
            connection_store,
        };

        this.register_subscriptions(window, cx);

        this
    }

    fn register_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.subscribe_in(
            &self.name_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.form_errors.remove("name");
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.host_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.form_errors.remove("host");
                    cx.notify();
                }
            },
        )
        .detach();

        // Port: real-time validation — chỉ cho phép số
        cx.subscribe_in(
            &self.port_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    let val = this.port_input.read(cx).value();
                    if !val.is_empty() && val.chars().any(|c| !c.is_ascii_digit()) {
                        this.form_errors
                            .insert("port", "Port chỉ được nhập số".into());
                    } else {
                        this.form_errors.remove("port");
                    }
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.user_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.form_errors.remove("user");
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.database_input,
            window,
            |this, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.form_errors.remove("database");
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.kind_select,
            window,
            |this, _select, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    let kind: DatabaseKind = match value.as_ref() {
                        "SQLite" => DatabaseKind::Sqlite,
                        "PostgreSQL" => DatabaseKind::Postgres,
                        _ => return,
                    };
                    this.on_kind_changed(kind, window, cx);
                }
            },
        )
        .detach();
    }

    /// Validate tất cả required fields. Trả về `true` nếu form hợp lệ.
    fn validate_form(&mut self, cx: &mut Context<Self>) -> bool {
        self.form_errors.clear();

        let name = self.name_input.read(cx).value().to_string();
        if name.trim().is_empty() {
            self.form_errors
                .insert("name", "Tên kết nối không được để trống".into());
        }

        match self.selected_kind.category() {
            ConnectionCategory::NetworkBased => {
                let host = self.host_input.read(cx).value().to_string();
                if host.trim().is_empty() {
                    self.form_errors
                        .insert("host", "Host không được để trống".into());
                }

                let port_str = self.port_input.read(cx).value().to_string();
                if port_str.trim().is_empty() {
                    self.form_errors
                        .insert("port", "Port không được để trống".into());
                } else {
                    match port_str.trim().parse::<u16>() {
                        Ok(p) if p > 0 => {} // hợp lệ
                        _ => {
                            self.form_errors
                                .insert("port", "Port phải là số từ 1 đến 65535".into());
                        }
                    }
                }

                let user = self.user_input.read(cx).value().to_string();
                if user.trim().is_empty() {
                    self.form_errors
                        .insert("user", "User không được để trống".into());
                }

                let database = self.database_input.read(cx).value().to_string();
                if database.trim().is_empty() {
                    self.form_errors
                        .insert("database", "Database không được để trống".into());
                }
            }
            ConnectionCategory::FileBased => {
                if self.selected_path.is_none() {
                    self.form_errors
                        .insert("file", "Vui lòng chọn file SQLite".into());
                }
            }
        }

        self.form_errors.is_empty()
    }

    /// Xử lý sự kiện thay đổi loại Database để kết nối.
    fn on_kind_changed(&mut self, kind: DatabaseKind, window: &mut Window, cx: &mut Context<Self>) {
        self.form_errors.clear();

        let old_category = self.selected_kind.category();
        let new_category = kind.category();

        let default_port = kind.default_port().to_string();
        let default_user = kind.default_user().to_string();
        let default_database = kind.default_database().to_string();

        self.selected_kind = kind;

        if new_category == ConnectionCategory::NetworkBased {
            self.port_input.update(cx, |state, cx| {
                state.set_value(&default_port, window, cx);
            });
            self.user_input.update(cx, |state, cx| {
                state.set_value(&default_user, window, cx);
            });
            self.database_input.update(cx, |state, cx| {
                state.set_value(&default_database, window, cx);
            });
        }

        if old_category != new_category {}

        cx.notify();
    }

    /// Xử lý sự kiện chọn file SQLite.
    fn on_browse_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let dialog = rfd::AsyncFileDialog::new()
                .set_title("Chọn file SQLite")
                .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                .add_filter("All Files", &["*"]);

            if let Some(file) = dialog.pick_file().await {
                let path = file.path().to_string_lossy().to_string();
                this.update(cx, |this, _cx| {
                    this.selected_path = Some(path);
                })
                .ok();
            }
        })
        .detach();
    }

    fn get_connection_config(&self, cx: &mut Context<Self>) -> DatabaseConfig {
        match self.selected_kind.category() {
            ConnectionCategory::FileBased => match self.selected_path.clone() {
                Some(path) => DatabaseConfig::sqlite(format!("sqlite:{}", path)),
                None => DatabaseConfig::sqlite("sqlite::memory:"),
            },
            ConnectionCategory::NetworkBased => {
                let read_val = |input: &Entity<InputState>, cx: &App| {
                    let v = input.read(cx).value().to_string();
                    (!v.is_empty()).then_some(v)
                };

                let host = read_val(&self.host_input, cx).unwrap_or_else(|| "localhost".into());
                let user = read_val(&self.user_input, cx)
                    .unwrap_or_else(|| self.selected_kind.default_user().to_string());
                let password = read_val(&self.password_input, cx).unwrap_or_default();
                let database = read_val(&self.database_input, cx).unwrap_or_default();

                let port: u16 = self
                    .port_input
                    .read(cx)
                    .value()
                    .parse()
                    .unwrap_or_else(|_p| self.selected_kind.default_port());

                DatabaseConfig::network(
                    self.selected_kind.clone(),
                    &host,
                    port,
                    &user,
                    &password,
                    &database,
                )
            }
        }
    }

    /// Xử lý sự kiện test kết nối
    fn on_test(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let config = self.get_connection_config(cx);
        self.is_testing = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = SqlClient::test_connection(config).await;
            this.update(cx, |this, cx| {
                this.is_testing = false;
                match result {
                    Ok(_) => tracing::info!("Kết nối thành công"),
                    Err(e) => tracing::error!("Lỗi kết nối: {e}"),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Xử lý sự kiện submit form. Nếu form không hợp lệ, không làm gì cả.
    fn on_submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.validate_form(cx) {
            cx.notify();
            return;
        }

        let name = self.name_input.read(cx).value().to_string();
        let config = self.get_connection_config(cx);

        self.connection_store.update(cx, |store, cx| {
            store.add_connection(name.into(), config, cx);
        });

        window.remove_window();
    }

    fn on_cancel(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }

    fn render_network_fields(&self) -> impl IntoElement {
        let kind = &self.selected_kind;

        let host_error = self.form_errors.get("host").cloned();
        let port_error = self.form_errors.get("port").cloned();
        let user_error = self.form_errors.get("user").cloned();
        let database_error = self.form_errors.get("database").cloned();

        let host_desc: SharedString = "Địa chỉ máy chủ".into();
        let port_desc: SharedString = format!("Mặc định: {}", kind.default_port()).into();
        let user_desc: SharedString = "Tên người dùng kết nối".into();
        let db_desc: SharedString = format!("Mặc định: {}", kind.default_database()).into();

        // Mình không rõ là cái hàm này nó tối ưu không nữa khi nó sử dụng .to_string()
        let description_fn = |err: &Option<SharedString>, default: &SharedString| -> Div {
            if let Some(msg) = err {
                div().text_color(red()).child(msg.clone())
            } else {
                div().child(default.clone())
            }
        };

        h_form()
            .label_width(px(100.))
            .child(
                field()
                    .label("Host")
                    .required(true)
                    .description_fn(move |_, _| description_fn(&host_error, &host_desc))
                    .child(Input::new(&self.host_input)),
            )
            .child(
                field()
                    .label("Port")
                    .required(true)
                    .description_fn(move |_, _| description_fn(&port_error, &port_desc))
                    .child(Input::new(&self.port_input)),
            )
            .child(
                field()
                    .label("User")
                    .required(true)
                    .description_fn(move |_, _| description_fn(&user_error, &user_desc))
                    .child(Input::new(&self.user_input)),
            )
            .child(
                field()
                    .label("Mật khẩu")
                    .description("Để trống nếu không có")
                    .child(Input::new(&self.password_input).mask_toggle()),
            )
            .child(
                field()
                    .label("Database")
                    .required(true)
                    .description_fn(move |_, _| description_fn(&database_error, &db_desc))
                    .child(Input::new(&self.database_input)),
            )
    }

    fn render_file_fields(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let path_display: SharedString = self
            .selected_path
            .as_deref()
            .unwrap_or("Choose a file...")
            .into();

        let file_error = self.form_errors.get("file").cloned();

        h_form().label_width(px(100.)).child(
            field()
                .label("File")
                .required(true)
                .description_fn(move |_, _| {
                    if let Some(msg) = &file_error {
                        div().text_color(red()).child(msg.clone())
                    } else {
                        div().child("Đường dẫn tới file SQLite (.db, .sqlite, .sqlite3)")
                    }
                })
                .child(
                    h_flex()
                        .gap_2()
                        .flex_1()
                        .child(
                            h_flex()
                                .flex_1()
                                .border_1()
                                .h_8()
                                .px_3()
                                .overflow_x_scrollbar()
                                .items_center()
                                .border_color(cx.theme().border)
                                .rounded(cx.theme().radius)
                                .text_sm()
                                .when(self.selected_path.is_none(), |this| {
                                    this.text_color(cx.theme().muted_foreground)
                                })
                                .child(path_display),
                        )
                        .child(
                            Button::new("btn-browse")
                                .outline()
                                .label("Browse")
                                .on_click(cx.listener(|this: &mut Self, _, window, cx| {
                                    this.on_browse_file(window, cx);
                                })),
                        ),
                ),
        )
    }
}

impl Render for ConnectionWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let scroll_handle = self.scroll_handle.clone();
        let kind = self.selected_kind.clone();

        let form_content = match kind.category() {
            ConnectionCategory::NetworkBased => self.render_network_fields().into_any_element(),
            ConnectionCategory::FileBased => self.render_file_fields(cx).into_any_element(),
        };

        v_flex()
            .id("connection-window")
            .size_full()
            .bg(theme.background)
            .child(
                v_flex()
                    .bg(cx.theme().secondary)
                    .border_b_1()
                    .border_color(theme.border)
                    .pb_2()
                    .child(self.title_bar.clone())
                    .child(
                        v_flex()
                            .px_6()
                            .pb_2()
                            .gap_4()
                            .child(
                                field()
                                    .label("Loại Database")
                                    .required(true)
                                    .description("Chọn loại database cần kết nối")
                                    .child(Select::new(&self.kind_select)),
                            )
                            .child({
                                let name_error = self.form_errors.get("name").cloned();
                                field()
                                    .label("Tên kết nối")
                                    .required(true)
                                    .description_fn(move |_, _| {
                                        if let Some(err) = &name_error {
                                            div().text_color(red()).child(err.clone())
                                        } else {
                                            div()
                                        }
                                    })
                                    .child(Input::new(&self.name_input))
                            }),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .id("connection-form-scroll")
                            .track_scroll(&scroll_handle)
                            .overflow_y_scroll()
                            .size_full()
                            .pt_4()
                            .pb_2()
                            .px_6()
                            .child(form_content),
                    )
                    .vertical_scrollbar(&scroll_handle),
            )
            .child(
                h_flex()
                    .flex_shrink_0()
                    .gap_1()
                    .px_6()
                    .py_4()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .child(
                        Button::new("btn-test")
                            .link()
                            .small()
                            .label(if self.is_testing {
                                "Đang kiểm tra..."
                            } else {
                                "Kiểm tra kết nối"
                            })
                            .loading(self.is_testing)
                            .disabled(self.is_testing)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.on_test(window, cx);
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("btn-cancel")
                            .outline()
                            .cursor_pointer()
                            .label("Hủy")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.on_cancel(window, cx);
                            })),
                    )
                    .child(
                        Button::new("btn-connect")
                            .primary()
                            .label("Kết nối")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.on_submit(window, cx);
                            })),
                    ),
            )
    }
}
