use std::marker::PhantomData;

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

impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> SessionBackend<T>
    for MongoBackend<T>
{
    fn login(&self, _name: String, _password: String) -> Result<Session, SessionError> {
        todo!()
    }

    fn validate(&self, _session: Session) -> std::result::Result<T, SessionError> {
        todo!()
    }
}

impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> IdentityBackend<T>
    for MongoBackend<T>
{
    fn get_all(&self) -> Result<Vec<T>, IdentityError> {
        todo!()
    }

    fn create(&self, _identity: T) -> Result<(), IdentityError> {
        todo!()
    }

    fn get_by_id(&self, _id: String) -> Result<T, IdentityError> {
        todo!()
    }

    fn update_by_id(&self, _id: String, _identity: T) -> Result<(), IdentityError> {
        todo!()
    }

    fn delete_by_id(&self, _id: String) -> Result<(), IdentityError> {
        todo!()
    }
}
