use actix_web::{
    Scope,
    web::{Data, ServiceConfig},
};
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
    path_prefix: Option<String>,
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

    /// Use this if you want to extend an existing scope with identity and session apis. Use configure if they should have their own path prefix.
    /// Example:
    ///     Given: scope set to '/api'
    ///     Then adds: '/api/identity/...' and '/api/session/...' apis
    pub fn configure_with_scope(self, scope: Scope) -> Scope {
        let data = Data::new(self.clone());
        let scope = scope.app_data(data);

        let scope = scope.configure(|cfg| self.configure_services(cfg));

        scope
    }

    fn configure_services(self, cfg: &mut ServiceConfig) {
        let data = Data::new(self);

        cfg.configure(|cfg| data.clone().identity_provider.configure(cfg))
            .configure(|cfg| data.clone().session_provider.configure(cfg));
    }

    /// Use this if identity and session apis should have their own path prefix. Use configure_with_scope if you want to extend an existing scope.
    /// Example:
    ///     Given: cfg set to '/' and path_prefix = "test"
    ///     Then adds: '/test/identity/...' and '/test/session/...' apis
    pub fn configure(self, cfg: &mut ServiceConfig) {
        let data = Data::new(self.clone());

        let scope = Scope::new(&self.path_prefix.clone().unwrap_or("".into()))
            .configure(|cfg| self.configure_services(cfg));

        cfg.app_data(data.clone()).service(scope);
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
    path_prefix: Option<String>,
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
            path_prefix: None,
        }
    }

    pub fn set_identity_path(self, path: String) -> Self {
        let mut self_mut = self;

        self_mut.identity_provider.identity_base_path = path;

        self_mut
    }

    pub fn set_login_path(self, path: String) -> Self {
        let mut self_mut = self;

        self_mut.session_provider.login_path = path;

        self_mut
    }

    pub fn set_validate_path(self, path: String) -> Self {
        let mut self_mut = self;

        self_mut.session_provider.validate_path = path;

        self_mut
    }

    pub fn set_path_prefix(self, path: Option<String>) -> Self {
        let mut self_mut = self;

        self_mut.path_prefix = path;

        self_mut
    }

    pub fn build(self) -> AuthProvider<T, J> {
        AuthProvider {
            _backend: Data::new(self.backend),
            session_provider: Data::new(self.session_provider),
            identity_provider: Data::new(self.identity_provider),
            path_prefix: self.path_prefix,
        }
    }
}
