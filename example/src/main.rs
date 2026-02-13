use actix_web::{App, HttpServer};
use serde::{Deserialize, Serialize};
use toro_auth_core::provider::AuthProvider;
use toro_auth_mongo::MongoBackend;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let identity = AuthProvider::default_with_backend(MongoBackend::<User>::new());

    HttpServer::new(move || App::new().configure(|cfg| identity.clone().configure(cfg)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    name: String,
}
