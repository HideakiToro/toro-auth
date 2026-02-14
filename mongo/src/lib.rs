use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{Client, Collection, Database, bson::doc};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use toro_auth_core::{
    identity::{IdentityBackend, IdentityError},
    session::{Session, SessionBackend, SessionError},
};
use uuid::Uuid;

pub trait ObjectId {
    fn id(&self) -> Uuid;
    fn set_id(&mut self, id: Uuid);
}

#[derive(Debug)]
pub enum MongoInitError {
    FailedToConnect,
}

#[derive(Clone)]
pub struct MongoBackend<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
> {
    _mapper: PhantomData<T>,
    identity_db: Collection<T>,
    session_db: Collection<Session>,
}

impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    MongoBackend<T>
{
    pub fn new(db: Database) -> Self {
        Self {
            _mapper: PhantomData,
            identity_db: db.collection("identity"),
            session_db: db.collection("session"),
        }
    }

    pub async fn from_url(url: String, db_name: String) -> Result<Self, MongoInitError> {
        let client = Client::with_uri_str(url)
            .await
            .map_err(|_| MongoInitError::FailedToConnect)?;
        let db = client.database(&db_name);
        Ok(Self::new(db))
    }

    pub async fn search_identity(&self, username: String) -> Result<Vec<T>, IdentityError> {
        let mut res = match self
            .identity_db
            .find(doc! {
                "name": {
                    "$regex": username,
                    "$options": "i"
                }
            })
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(IdentityError::InternalServerError);
            }
        };

        let mut identities = Vec::new();
        while let Some(identity) = res.try_next().await.map_err(|e| {
            eprintln!("{e:#?}");
            IdentityError::InternalServerError
        })? {
            identities.push(identity);
        }

        Ok(identities)
    }
}

#[async_trait]
impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    SessionBackend<T> for MongoBackend<T>
{
    async fn login(&self, username: String, password: String) -> Result<Session, SessionError> {
        let res = match self
            .identity_db
            .find_one(doc! {
                "name": username,
                "password": password
            })
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(SessionError::InternalServerError);
            }
        };
        let Some(identity) = res else {
            return Err(SessionError::InvalidLogin);
        };

        let session = Session {
            id: Uuid::new_v4().into(),
            user_id: identity.id().into(),
        };

        let _ = match self.session_db.insert_one(session.clone()).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(SessionError::InternalServerError);
            }
        };

        Ok(session)
    }

    async fn validate(&self, session_id: String) -> Result<T, SessionError> {
        let res = match self
            .session_db
            .find_one(doc! {
                "id": {
                    "$eq": session_id
                }
            })
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(SessionError::InternalServerError);
            }
        };
        let Some(session) = res else {
            return Err(SessionError::InvalidOrMissingSession);
        };

        let identity = self
            .get_by_id(session.user_id)
            .await
            .map_err(|_| SessionError::InternalServerError)?;

        Ok(identity)
    }
}

#[async_trait]
impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    IdentityBackend<T> for MongoBackend<T>
{
    async fn get_all(&self) -> Result<Vec<T>, IdentityError> {
        let mut res = match self.identity_db.find(doc! {}).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(IdentityError::InternalServerError);
            }
        };

        let mut identities = Vec::new();
        while let Some(identity) = res.try_next().await.map_err(|e| {
            eprintln!("{e:#?}");
            IdentityError::InternalServerError
        })? {
            identities.push(identity);
        }

        Ok(identities)
    }

    async fn create(&self, mut identity: T) -> Result<(), IdentityError> {
        identity.set_id(Uuid::new_v4());

        self.identity_db
            .insert_one(identity.clone())
            .await
            .map_err(|e| {
                eprintln!("{e:#?}");
                IdentityError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_by_id(&self, id: String) -> Result<T, IdentityError> {
        let res = match self
            .identity_db
            .find_one(doc! {
                "id": {
                    "$eq": id
                }
            })
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(IdentityError::InternalServerError);
            }
        };
        let Some(identity) = res else {
            return Err(IdentityError::NotFound);
        };
        Ok(identity)
    }

    async fn update_by_id(&self, _id: String, _identity: T) -> Result<(), IdentityError> {
        todo!()
    }

    async fn delete_by_id(&self, id: String) -> Result<(), IdentityError> {
        let res = match self
            .identity_db
            .delete_one(doc! {
                "id": {
                    "$eq": id
                }
            })
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{e}");
                return Err(IdentityError::InternalServerError);
            }
        };

        match res.deleted_count {
            0 => Err(IdentityError::NotFound),
            _ => Ok(()),
        }
    }
}
