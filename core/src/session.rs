use actix_web::{
    Error, FromRequest, HttpResponse, Responder,
    cookie::{
        Cookie,
        time::{Duration, OffsetDateTime},
    },
    web::{Data, Json, ServiceConfig, get, post},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
}

pub enum SessionError {
    InvalidOrMissingSession,
    InternalServerError,
    ServiceUnavailable,
}

impl From<SessionError> for HttpResponse {
    fn from(value: SessionError) -> Self {
        match value {
            SessionError::InternalServerError => HttpResponse::InternalServerError().finish(),
            SessionError::InvalidOrMissingSession => HttpResponse::Unauthorized().finish(),
            SessionError::ServiceUnavailable => HttpResponse::ServiceUnavailable().finish(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Clone)]
pub struct SessionProvider<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    login_path: String,
    validate_path: String,
    backend: Data<Box<dyn SessionBackend<T>>>,
}

impl<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> SessionProvider<T> {
    pub fn default_with_backend(backend: Data<Box<dyn SessionBackend<T>>>) -> Self {
        Self {
            login_path: String::from("session/login"),
            validate_path: String::from("session/validate"),
            backend,
        }
    }

    pub fn configure(&self, cfg: &mut ServiceConfig) {
        cfg.route(&self.login_path, post().to(login::<T>))
            .route(&self.validate_path, get().to(validate::<T>));
    }

    pub async fn validate(&self, session: Session) -> Result<T, SessionError> {
        println!("Validating session {}...", session.id);
        self.backend.validate(session).await
    }

    pub async fn login(&self, username: String, password: String) -> Result<Session, SessionError> {
        println!("Loggin in as {username} with password {password}...");
        self.backend.login(username, password).await
    }
}

async fn validate<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>(
    session_provider: Data<SessionProvider<T>>,
    session: Data<Session>,
) -> impl Responder {
    match session_provider.validate(session.get_ref().clone()).await {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => e.into(),
    }
}

async fn login<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>(
    session_provider: Data<SessionProvider<T>>,
    request: Json<LoginRequest>,
) -> impl Responder {
    let request = request.0;
    let session = match session_provider
        .login(request.username, request.password)
        .await
    {
        Ok(session) => session,
        Err(e) => return e.into(),
    };

    let session_cookie = Cookie::build("sessionId", session.id)
        .path("/")
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(10)))
        .finish();
    HttpResponse::Ok().cookie(session_cookie).finish()
}

impl FromRequest for Session {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let Some(session_id) = req.cookie("sessionId") else {
                return Ok(Self {
                    id: "invalid".into(),
                });
            };

            Ok(Self {
                id: session_id.value().into(),
            })
        })
    }
}

#[async_trait]
pub trait SessionBackend<T>: Send + Sync {
    async fn validate(&self, session: Session) -> Result<T, SessionError>;
    async fn login(&self, username: String, password: String) -> Result<Session, SessionError>;
}
