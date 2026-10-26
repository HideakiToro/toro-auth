use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSON;
use sqlx::{FromRow, Pool, Postgres, migrate, query, query_as};
use std::{marker::PhantomData, str::FromStr};
use toro_auth_core::{
    ObjectId,
    identity::{IdentityBackend, IdentityError},
    session::{Session, SessionBackend, SessionError},
};
use uuid::Uuid;

#[derive(Debug)]
pub enum PostgresInitError {
    FailedToConnect,
    FailedToCreateTables,
}

#[derive(Clone)]
pub struct PostgresBackend<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
> {
    _mapper: PhantomData<T>,
    pool: Pool<Postgres>,
}

impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    PostgresBackend<T>
{
    pub fn new(connection_pool: Pool<Postgres>) -> Self {
        Self {
            _mapper: PhantomData,
            pool: connection_pool,
        }
    }

    pub async fn create_tables(&self) -> Result<(), PostgresInitError> {
        migrate!().run(&self.pool).await.map_err(|err| {
            eprintln!("Failed to create tables: {err:#?}");
            PostgresInitError::FailedToCreateTables
        })?;
        Ok(())
    }

    pub async fn from_url(url: String) -> Result<Self, PostgresInitError> {
        let pool = match Pool::connect(&url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Failed to connect to postgres: {e:#?}");
                return Err(PostgresInitError::FailedToConnect);
            }
        };
        Ok(Self::new(pool))
    }

    pub async fn search_identity(&self, username: String) -> Result<Vec<T>, IdentityError> {
        let res: Vec<IdentityRow> =
            match query_as("SELECT * FROM toroauth.identity WHERE data ->>'username' ~* $1")
                .bind(username)
                .fetch_all(&self.pool)
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Failed to search identities: {e}");
                    return Err(IdentityError::InternalServerError);
                }
            };

        let mut identities = Vec::new();
        for identity_row in res {
            match serde_json::from_value::<T>(identity_row.data) {
                Ok(val) => {
                    identities.push(val);
                }
                Err(e) => {
                    eprintln!("Invalid value while searching identities: {e:#?}");
                    continue;
                }
            }
        }

        Ok(identities)
    }
}

#[async_trait]
impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    SessionBackend<T> for PostgresBackend<T>
{
    async fn login(&self, username: String, password: String) -> Result<Session<T>, SessionError> {
        let res: T = match query_as::<Postgres, IdentityRow>("SELECT * FROM toroauth.identity WHERE data->>'username' = $1 AND data->>'password' = $2 LIMIT 1").bind(username).bind(password).fetch_optional(&self.pool).await {
            Ok(Some(res)) => serde_json::from_value(res.data).map_err(|err| {
                eprintln!("Failed to deserialize identity on login: {err:#?}");
                SessionError::InternalServerError
            })?,
            Ok(None) => {
                eprintln!("Username-Password-Combination do not match any known identity");
                return Err(SessionError::UserNotFound);
            }
            Err(e) => {
                eprintln!("Failed to query for identity on login: {e}");
                return Err(SessionError::InternalServerError);
            }
        };

        let Some(user_id) = res.id() else {
            return Err(SessionError::InternalServerError);
        };

        let session_id = Uuid::new_v4();
        let session = Session::new(session_id.into(), user_id.into());

        query("INSERT INTO toroauth.session (id, user_id) VALUES ($1, $2)")
            .bind(session_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                eprintln!("Failed to create session on login: {err}");
                SessionError::InternalServerError
            })?;

        Ok(session)
    }

    async fn validate(&self, session_id: String) -> Result<T, SessionError> {
        let session_id = Uuid::from_str(&session_id).map_err(|err| {
            eprintln!("Failed to parse session-id into uuid: {err:#?}");
            SessionError::InvalidOrMissingSession
        })?;
        let res: Option<SessionRow> =
            query_as("SELECT * FROM toroauth.session WHERE id = $1 LIMIT 1")
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|err| {
                    eprintln!("Failed to query for session: {err:#?}");
                    SessionError::InternalServerError
                })?;

        let Some(row) = res else {
            eprintln!("Session-Id does not match any known Session");
            return Err(SessionError::InvalidOrMissingSession);
        };

        let identity = self.get_by_id(row.user_id.into()).await.map_err(|_| {
            eprintln!("Couldn't find user related to session");
            SessionError::UserNotFound
        })?;

        Ok(identity)
    }
}

#[async_trait]
impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    IdentityBackend<T> for PostgresBackend<T>
{
    async fn get_all(&self) -> Result<Vec<T>, IdentityError> {
        let res: Vec<IdentityRow> = match query_as("SELECT * FROM toroauth.identity")
            .fetch_all(&self.pool)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Failed to get all identities: {e}");
                return Err(IdentityError::InternalServerError);
            }
        };

        let mut identities = Vec::new();
        for identity_row in res {
            match serde_json::from_value::<T>(identity_row.data) {
                Ok(val) => {
                    identities.push(val);
                }
                Err(e) => {
                    eprintln!("Invalid value while getting all identities: {e:#?}");
                    continue;
                }
            }
        }

        Ok(identities)
    }

    async fn create(&self, mut identity: T) -> Result<(), IdentityError> {
        let user_id = Uuid::new_v4();

        identity.set_id(user_id);
        let json =
            serde_json::to_value(identity).map_err(|_| IdentityError::InternalServerError)?;

        query("INSERT INTO toroauth.identity (id, data) VALUES ($1, $2)")
            .bind(user_id)
            .bind(json)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("Failed to create identity: {e:#?}");
                IdentityError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_by_username(&self, username: String) -> Result<Option<T>, IdentityError> {
        let res: Option<IdentityRow> =
            match query_as("SELECT * FROM toroauth.identity WHERE data ->>'username' = $1 LIMIT 1")
                .bind(username)
                .fetch_optional(&self.pool)
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Failed to search identity by username: {e}");
                    return Err(IdentityError::InternalServerError);
                }
            };

        let Some(row) = res else { return Ok(None) };

        let identity = serde_json::from_value(row.data).map_err(|err| {
            eprintln!("Failed to deserialize result from username query: {err}");
            IdentityError::InternalServerError
        })?;

        Ok(Some(identity))
    }

    async fn get_by_id(&self, id: String) -> Result<T, IdentityError> {
        let identity_id = Uuid::from_str(&id).map_err(|_| IdentityError::InvalidId)?;

        let res: Option<IdentityRow> =
            match query_as("SELECT * FROM toroauth.identity WHERE id = $1 LIMIT 1")
                .bind(identity_id)
                .fetch_optional(&self.pool)
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Failed to query identity by id: {e}");
                    return Err(IdentityError::InternalServerError);
                }
            };

        let Some(row) = res else {
            return Err(IdentityError::NotFound);
        };

        let identity = serde_json::from_value(row.data).map_err(|err| {
            eprintln!("Failed to deserialize result from id query: {err}");
            IdentityError::InternalServerError
        })?;

        Ok(identity)
    }

    async fn update_by_id(&self, id: String, identity: T) -> Result<(), IdentityError> {
        let identity_id = Uuid::from_str(&id).map_err(|_| IdentityError::InvalidId)?;

        let mut identity = identity;
        identity.set_id(identity_id);

        let json = serde_json::to_value(identity).map_err(|err| {
            eprintln!("Failed to serialize idenity for update: {err:#?}");
            IdentityError::InternalServerError
        })?;

        let res = match query("UPDATE toroauth.identity SET data = $1 WHERE id = $2")
            .bind(json)
            .bind(identity_id)
            .execute(&self.pool)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Failed to update identity: {e:#?}");
                return Err(IdentityError::InternalServerError);
            }
        };

        if res.rows_affected() <= 0 {
            return Err(IdentityError::NotFound);
        }

        Ok(())
    }

    async fn delete_by_id(&self, id: String) -> Result<(), IdentityError> {
        let identity_id = Uuid::from_str(&id).map_err(|_| IdentityError::InvalidId)?;

        match query("DELETE FROM toroauth.session WHERE user_id = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to remove sessions: {e:#?}");
                return Err(IdentityError::InternalServerError);
            }
        };

        let res = match query("DELETE FROM toroauth.identity WHERE id = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Failed to remove user: {e:#?}");
                return Err(IdentityError::InternalServerError);
            }
        };

        if res.rows_affected() <= 0 {
            return Err(IdentityError::NotFound);
        }

        Ok(())
    }
}

#[derive(Debug, FromRow)]
struct IdentityRow {
    #[allow(dead_code)]
    id: Uuid,
    pub data: JSON,
}

#[derive(Debug, FromRow)]
struct SessionRow {
    #[allow(dead_code)]
    id: Uuid,
    pub user_id: Uuid,
}
