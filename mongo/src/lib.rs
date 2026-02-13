use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use toro_auth_core::{
    identity::{IdentityBackend, IdentityError},
    session::{Session, SessionBackend, SessionError},
};

#[derive(Clone)]
pub struct MongoBackend<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> {
    _mapper: PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> MongoBackend<T> {
    pub fn new() -> Self {
        Self {
            _mapper: PhantomData,
        }
    }
}

#[async_trait]
impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> SessionBackend<T>
    for MongoBackend<T>
{
    async fn login(&self, _name: String, _password: String) -> Result<Session, SessionError> {
        todo!()
    }

    async fn validate(&self, _session: Session) -> std::result::Result<T, SessionError> {
        todo!()
    }
}

#[async_trait]
impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> IdentityBackend<T>
    for MongoBackend<T>
{
    async fn get_all(&self) -> Result<Vec<T>, IdentityError> {
        todo!()
    }

    async fn create(&self, _identity: T) -> Result<(), IdentityError> {
        todo!()
    }

    async fn get_by_id(&self, _id: String) -> Result<T, IdentityError> {
        todo!()
    }

    async fn update_by_id(&self, _id: String, _identity: T) -> Result<(), IdentityError> {
        todo!()
    }

    async fn delete_by_id(&self, _id: String) -> Result<(), IdentityError> {
        todo!()
    }
}
