use actix_web::web::{Data, ServiceConfig};
use serde::{Deserialize, Serialize};

use crate::{
    IntoPublic, ObjectId,
    identity::{IdentityBackend, IdentityProvider},
    session::{SessionBackend, SessionError, SessionProvider},
};

#[derive(Clone)]
pub struct AuthProvider<T, J>
where
    T: IntoPublic
        + ObjectId
        + Serialize
        + for<'de> Deserialize<'de>
        + Clone
        + Send
        + Sync
        + 'static,
    J: SessionBackend<T> + IdentityBackend<T> + Clone + Send + Sync + 'static,
{
    pub session_provider: Data<SessionProvider<T>>,
    pub identity_provider: Data<IdentityProvider<T>>,
    _backend: Data<J>,
}

impl<
    T: IntoPublic + ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
    J: SessionBackend<T> + IdentityBackend<T> + Clone + Send + Sync + 'static,
> AuthProvider<T, J>
{
    pub fn builder(backend: J) -> AuthProviderBuilder<T, J> {
        AuthProviderBuilder::default_with_backend(backend)
    }

    pub fn default_with_backend(backend: J) -> Self {
        AuthProviderBuilder::default_with_backend(backend).build()
    }

    pub fn configure(self, cfg: &mut ServiceConfig) {
        let data = Data::new(self);
        cfg.app_data(data.clone())
            .configure(|cfg| data.clone().identity_provider.configure(cfg))
            .configure(|cfg| data.clone().session_provider.configure(cfg));
    }

    pub async fn validate_session(&self, session_id: String) -> Result<T, SessionError> {
        self.session_provider.validate(session_id).await
    }
}

pub struct AuthProviderBuilder<T, J>
where
    T: IntoPublic
        + ObjectId
        + Serialize
        + for<'de> Deserialize<'de>
        + Clone
        + Send
        + Sync
        + 'static,
    J: SessionBackend<T> + IdentityBackend<T> + Send + Sync + 'static,
{
    session_provider: SessionProvider<T>,
    identity_provider: IdentityProvider<T>,
    backend: J,
}

impl<
    T: IntoPublic + ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
    J: SessionBackend<T> + IdentityBackend<T> + Clone + Send + Sync + 'static,
> AuthProviderBuilder<T, J>
{
    pub fn default_with_backend(backend: J) -> Self {
        Self {
            session_provider: SessionProvider::<T>::default_with_backend(Data::new(Box::new(
                backend.clone(),
            ))),
            identity_provider: IdentityProvider::<T>::default_with_backend(Data::new(Box::new(
                backend.clone(),
            ))),
            backend,
        }
    }

    pub fn build(self) -> AuthProvider<T, J> {
        AuthProvider {
            _backend: Data::new(self.backend),
            session_provider: Data::new(self.session_provider),
            identity_provider: Data::new(self.identity_provider),
        }
    }
}
