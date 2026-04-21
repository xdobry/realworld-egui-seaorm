use core::{api::{UICommand, UIResult}, users::{api::{UserCommand, UserResult}, dto::{LoginResponse, LoginUser}}};

use command_bus::{CommandBus, UIBus};

use crate::app::FormsApp;

pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub msg: Option<String>,
    event_bus: UIBus,
    first_frame: bool,
}

pub enum LoginAction {
    LoggedIn(LoginResponse),
    Cancel,
    None,
}

impl LoginForm {
    
    pub fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> LoginAction {
        let mut login_action = LoginAction::None;
        ui.add_space(80.0);
        ui.vertical_centered(|ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(300.0);
                ui.heading("Login");
                ui.add_space(10.0);
                ui.label("Email");
                let email_id = ui.make_persistent_id("login_email");
                let email_response = ui.add(
                    egui::TextEdit::singleline(&mut self.email)
                        .id(email_id)
                        .hint_text("Email"),
                );
                if self.first_frame {
                    email_response.request_focus();
                    self.first_frame = false;
                }
                ui.label("Password");
                let password_response = ui.add(
                egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Enter password")
                );
                let mut process_login = false;
                if password_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    process_login = true;
                }
                if let Some(msg) = &self.msg {
                    ui.strong(msg);
                }
                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    if ui.button("Login").clicked() {
                        process_login = true;
                    }
                    if ui.button("Cancel").clicked() {
                        login_action = LoginAction::Cancel;
                    }
                });
                if process_login {
                    self.event_bus.send_task(tx, UICommand::User(UserCommand::Login(LoginUser {
                        email: self.email.clone(),
                        password: self.password.clone(),
                    })));
                }
            });
        });
        if let Ok(msg) = self.event_bus.try_recv() {
            match msg {
                UIResult::DbError(msg) => {
                    self.msg = Some(msg);
                },
                UIResult::User(UserResult::LoginFailed(msg)) => {
                    self.msg = Some(msg);
                }
                UIResult::User(UserResult::Login(login_response)) => {
                    login_action = LoginAction::LoggedIn(login_response);
                }
                _ => {

                }
            }
        }
        login_action
    }

    pub fn new() -> Self {
        Self {
            email: "".into(),
            password: "".into(),
            event_bus: UIBus::default(),
            msg: None,
            first_frame: true,
        } 
    }
}