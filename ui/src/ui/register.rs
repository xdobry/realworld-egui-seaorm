use core::{api::{UICommand, UIResult}, users::{api::{UserCommand, UserResult}, dto::{LoginResponse, LoginUser, RegisterUser}}};

use command_bus::{CommandBus, UIBus};
use egui::Color32;

pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
    pub password2: String,
    pub msg: Option<String>,
    event_bus: UIBus,
    first_frame: bool,
}

pub enum RegisterAction {
    LoggedIn(LoginResponse),
    Cancel,
    None,
}

impl RegisterForm {
    
    pub fn show(&mut self, ui: &mut egui::Ui, tx: &mut CommandBus) -> RegisterAction {
        let mut login_action = RegisterAction::None;
        ui.add_space(80.0);
        ui.vertical_centered(|ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(300.0);
                ui.heading("Register User");
                ui.add_space(10.0);
                ui.label("Name");
                let name_id = ui.make_persistent_id("register_name");
                let name_response = ui.add(
                    egui::TextEdit::singleline(&mut self.name)
                        .id(name_id)
                        .hint_text("Name"),
                );
                if self.first_frame {
                    name_response.request_focus();
                    self.first_frame = false;
                }
                ui.label("Email");
                let _email_response = ui.add(
                    egui::TextEdit::singleline(&mut self.email)
                        .hint_text("Email"),
                );
                ui.label("Password");
                let _password_response = ui.add(
                egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Enter password")
                );
                ui.label("Password Repeat");
                let password_response = ui.add(
                egui::TextEdit::singleline(&mut self.password2)
                        .password(true)
                        .hint_text("Enter password")
                );
                let mut process_register = false;
                if password_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    process_register = true;
                }
                if let Some(msg) = &self.msg {
                    ui.colored_label(Color32::RED, msg);
                }
                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    if ui.button("Register").clicked() {
                        process_register = true;
                    }
                    if ui.button("Cancel").clicked() {
                        login_action = RegisterAction::Cancel;
                    }
                });
                if process_register {
                    if self.validate() {
                        self.event_bus.send_task(tx, UICommand::User(UserCommand::Register(RegisterUser {
                            email: self.email.clone(),
                            name: self.name.clone(),
                            password: self.password.clone(),
                        })));
                    }
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
                    login_action = RegisterAction::LoggedIn(login_response);
                }
                _ => {

                }
            }
        }
        login_action
    }

    pub fn validate(&mut self) -> bool {
        if self.name.len()==0 {
            self.msg = Some("Name must be set".into());
            return false;
        }
        if self.email.len()==0 {
            self.msg = Some("Email must be set".into());
            return false;
        }
        if self.password.len()==0 {
            self.msg = Some("Password must be set".into());
            return false;
        }
        if self.password != self.password2 {
            self.msg = Some("Password repeat must be same".into());
            return false;
        }
        true
    }

    pub fn new() -> Self {
        Self {
            name: "".into(),
            email: "".into(),
            password: "".into(),
            password2: "".into(),
            event_bus: UIBus::default(),
            msg: None,
            first_frame: true,
        } 
    }
}