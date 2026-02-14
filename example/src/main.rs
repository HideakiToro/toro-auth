use actix_web::{App, HttpServer};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use toro_auth_core::{IntoPublic, ObjectId, provider::AuthProvider};
use toro_auth_mongo::MongoBackend;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let identity = AuthProvider::default_with_backend(
        MongoBackend::<DBUser>::from_url("mongodb://localhost:27017".into(), "example".into())
            .await
            .unwrap(),
    );

    HttpServer::new(move || App::new().configure(|cfg| identity.clone().configure(cfg)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Serialize, Deserialize, Clone)]
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
