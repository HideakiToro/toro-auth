use actix_web::{App, HttpServer, Scope};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use toro_auth_core::{IntoPublic, ObjectId, provider::AuthProvider};
use toro_auth_postgres::PostgresBackend;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let postgres_backend =
        PostgresBackend::<DBUser>::from_url("postgres://postgres:admin@192.168.2.105:5432".into())
            .await
            .unwrap();
    postgres_backend.create_tables().await.unwrap();
    let postgres_identity = AuthProvider::default_with_backend(postgres_backend);

    HttpServer::new(move || {
        let full_scope = Scope::new("");
        let api_scope = Scope::new("api/v1");

        let (api_scope, full_scope) = postgres_identity
            .clone()
            .configure_with_scope(api_scope, Some(full_scope));
        let full_scope = full_scope.expect("Should be Some, since Some was passed.");

        let full_scope = full_scope.service(api_scope);
        App::new().service(full_scope)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[derive(Serialize, Clone)]
struct User {
    username: String,
    id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DBUser {
    username: String,
    password: String,
    id: Option<String>,
}

impl ObjectId for DBUser {
    fn id(&self) -> Option<Uuid> {
        Uuid::from_str(&self.id.clone()?).ok()
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = Some(id.into());
    }

    fn username(&self) -> String {
        self.username.clone()
    }
}

impl IntoPublic for DBUser {
    type Public = User;
    fn into_public(self) -> Self::Public {
        Self::Public {
            username: self.username,
            id: self.id,
        }
    }
}
