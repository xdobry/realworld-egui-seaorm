use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};
use models::entity::users::Model;
use crate::{api::AuthContext, dto::ChangeRecord, users::dto::{LoginResponse, LoginUser, RegisterUser}};


#[derive(Serialize, Deserialize, Debug)]
pub enum UserCommand {
    Reload,
    Create(Model),
    Update(ChangeRecord),
    Load(Uuid),
    Delete(Uuid),
    Login(LoginUser),
    Register(RegisterUser),
}

impl UserCommand {
    pub fn has_access(&self, auth_context: &AuthContext) -> bool {
        match self {
            UserCommand::Reload | UserCommand::Load(_) | UserCommand::Login(_) | UserCommand::Register(_) => {
                true
            }
            _ => {
                !auth_context.is_anonymous()
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub enum UserResult {
    Users(Vec<Model>),
    User(Model),
    Login(LoginResponse),
    LoginFailed(String),
}