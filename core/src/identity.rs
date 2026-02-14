use actix_web::{
    HttpResponse, Responder,
    web::{Data, Json, Path, ServiceConfig, delete, get, post, put},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ObjectId;

pub enum IdentityError {
    NotFound,
    InternalServerError,
    ServiceUnavailable,
    Unauthorized,
}

impl From<IdentityError> for HttpResponse {
    fn from(value: IdentityError) -> Self {
        match value {
            IdentityError::InternalServerError => HttpResponse::InternalServerError().finish(),
            IdentityError::NotFound => HttpResponse::NotFound().finish(),
            IdentityError::ServiceUnavailable => HttpResponse::ServiceUnavailable().finish(),
            IdentityError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}

#[derive(Deserialize)]
pub struct IdentityGetPath {
    id: String,
}

#[derive(Clone)]
pub struct IdentityProvider<T>
where
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    identity_base_path: String,
    backend: Data<Box<dyn IdentityBackend<T>>>,
}

impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    IdentityProvider<T>
{
    pub fn default_with_backend(backend: Data<Box<dyn IdentityBackend<T>>>) -> Self {
        Self {
            identity_base_path: String::from("identity"),
            backend,
        }
    }

    pub fn configure(&self, cfg: &mut ServiceConfig) {
        cfg.route(&self.identity_base_path, get().to(get_all::<T>))
            .route(&self.identity_base_path, post().to(create::<T>))
            .route(
                &format!("{}/{{id}}", self.identity_base_path),
                get().to(get_by_id::<T>),
            )
            .route(
                &format!("{}/{{id}}", self.identity_base_path),
                put().to(update_by_id::<T>),
            )
            .route(
                &format!("{}/{{id}}", self.identity_base_path),
                delete().to(delete_by_id::<T>),
            );
    }

    pub async fn get_all(&self) -> Result<Vec<T>, IdentityError> {
        println!("Getting all identities...");
        self.backend.get_all().await
    }

    pub async fn create(&self, identity: T) -> Result<(), IdentityError> {
        println!("Creating identity...");
        self.backend.create(identity).await
    }

    pub async fn get_by_id(&self, id: String) -> Result<T, IdentityError> {
        println!("Getting identity {id}...");
        self.backend.get_by_id(id).await
    }

    pub async fn update(&self, id: String, identity: T) -> Result<(), IdentityError> {
        println!("Updating identity {id}...");
        self.backend.update_by_id(id, identity).await
    }

    pub async fn delete(&self, id: String) -> Result<(), IdentityError> {
        println!("Deleting identity {id}...");
        self.backend.delete_by_id(id).await
    }
}

async fn get_all<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    identity_provider: Data<IdentityProvider<T>>,
) -> impl Responder {
    match identity_provider.get_all().await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => e.into(),
    }
}

async fn create<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    identity_provider: Data<IdentityProvider<T>>,
    identity: Json<T>,
) -> impl Responder {
    match identity_provider.create(identity.0).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => e.into(),
    }
}

async fn get_by_id<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    identity_provider: Data<IdentityProvider<T>>,
    path: Path<IdentityGetPath>,
) -> impl Responder {
    match identity_provider.get_by_id(path.id.clone()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => e.into(),
    }
}

async fn update_by_id<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    identity_provider: Data<IdentityProvider<T>>,
    path: Path<IdentityGetPath>,
    identity: Json<T>,
) -> impl Responder {
    match identity_provider.update(path.id.clone(), identity.0).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => e.into(),
    }
}

async fn delete_by_id<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    identity_provider: Data<IdentityProvider<T>>,
    path: Path<IdentityGetPath>,
) -> impl Responder {
    match identity_provider.delete(path.id.clone()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.into(),
    }
}

#[async_trait]
pub trait IdentityBackend<T>: Send + Sync
where
    T: ObjectId + Serialize + for<'de> Deserialize<'de>,
{
    async fn get_all(&self) -> Result<Vec<T>, IdentityError>;
    async fn create(&self, mut identity: T) -> Result<(), IdentityError>;
    async fn get_by_id(&self, id: String) -> Result<T, IdentityError>;
    async fn update_by_id(&self, id: String, identity: T) -> Result<(), IdentityError>;
    async fn delete_by_id(&self, id: String) -> Result<(), IdentityError>;
}
