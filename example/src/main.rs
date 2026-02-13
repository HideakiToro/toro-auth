use actix_web::{App, HttpServer};
use serde::{Deserialize, Serialize};
use toro_auth_core::{
    identity::IdentityBackend,
    provider::AuthProvider,
    session::{Session, SessionBackend, SessionError},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let identity = AuthProvider::default_with_backend(MockBackend {});

    HttpServer::new(move || App::new().configure(|cfg| identity.clone().configure(cfg)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    name: String,
}

#[derive(Clone)]
struct MockBackend {}

impl SessionBackend<User> for MockBackend {
    fn login(&self, _username: String, _password: String) -> Result<Session, SessionError> {
        Ok(Session { id: "".into() })
    }

    fn validate(
        &self,
        _session: toro_auth_core::session::Session,
    ) -> std::result::Result<User, toro_auth_core::session::SessionError> {
        Ok(User {
            name: "test".into(),
        })
    }
}

impl IdentityBackend<User> for MockBackend {
    fn get_all(&self) -> Result<Vec<User>, toro_auth_core::identity::IdentityError> {
        todo!()
    }

    fn create(&self, _identity: User) -> Result<(), toro_auth_core::identity::IdentityError> {
        todo!()
    }

    fn get_by_id(&self, _id: String) -> Result<User, toro_auth_core::identity::IdentityError> {
        todo!()
    }

    fn update_by_id(
        &self,
        _id: String,
        _identity: User,
    ) -> Result<(), toro_auth_core::identity::IdentityError> {
        todo!()
    }

    fn delete_by_id(&self, _id: String) -> Result<(), toro_auth_core::identity::IdentityError> {
        todo!()
    }
}
