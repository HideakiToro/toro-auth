use actix_web::{App, HttpServer};
use serde::{Deserialize, Serialize};
use toro_auth_core::{ObjectId, provider::AuthProvider};
use toro_auth_mongo::MongoBackend;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let identity = AuthProvider::default_with_backend(
        MongoBackend::<User>::from_url("mongo://localhost:27127".into(), "example".into())
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
    name: String,
    id: Option<Uuid>,
}

impl ObjectId for User {
    fn id(&self) -> Option<Uuid> {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = Some(id);
    }
}
