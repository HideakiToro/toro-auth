use actix_web::{
    FromRequest, HttpResponse, Responder,
    cookie::{
        Cookie,
        time::{Duration, OffsetDateTime},
    },
    http::StatusCode,
    web::{Data, Json, ServiceConfig, get, post},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, pin::Pin};

use crate::ObjectId;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session<T> {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    _mapped: Option<PhantomData<T>>,
}

impl<T> Session<T> {
    pub fn new(id: String, user_id: String) -> Self {
        Self {
            id,
            user_id,
            _mapped: None,
        }
    }
}

#[derive(Debug)]
pub enum SessionError {
    InvalidOrMissingSession,
    InternalServerError,
    ServiceUnavailable,
    InvalidLogin,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl From<SessionError> for HttpResponse {
    fn from(value: SessionError) -> Self {
        match value {
            SessionError::InternalServerError => HttpResponse::InternalServerError().finish(),
            SessionError::InvalidOrMissingSession | SessionError::InvalidLogin => {
                HttpResponse::Unauthorized().finish()
            }
            SessionError::ServiceUnavailable => HttpResponse::ServiceUnavailable().finish(),
        }
    }
}

impl actix_web::error::ResponseError for SessionError {
    fn status_code(&self) -> StatusCode {
        match self {
            SessionError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            SessionError::InvalidOrMissingSession | SessionError::InvalidLogin => {
                StatusCode::UNAUTHORIZED
            }
            SessionError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).finish()
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub struct SessionRes<T> {
    inner: T,
}

#[derive(Clone)]
pub struct SessionProvider<T>
where
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
{
    login_path: String,
    validate_path: String,
    backend: Data<Box<dyn SessionBackend<T>>>,
}

impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    SessionProvider<T>
{
    pub fn default_with_backend(backend: Data<Box<dyn SessionBackend<T>>>) -> Self {
        Self {
            login_path: String::from("session/login"),
            validate_path: String::from("session/validate"),
            backend,
        }
    }

    pub fn configure(&self, cfg: &mut ServiceConfig) {
        let data = Data::new(self.clone());
        cfg.app_data(data.clone())
            .route(&self.login_path, post().to(login::<T>))
            .route(&self.validate_path, get().to(validate::<T>));
    }

    pub async fn validate(&self, session_id: String) -> Result<T, SessionError> {
        println!("Validating session {session_id}...");
        self.backend.validate(session_id).await
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<Session<T>, SessionError> {
        println!("Loggin in as {username} with password {password}...");
        self.backend.login(username, password).await
    }
}

async fn validate<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    session: SessionRes<T>,
) -> impl Responder {
    HttpResponse::Ok().json(session.inner.clone())
}

async fn login<
    T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
>(
    session_provider: Data<SessionProvider<T>>,
    request: Json<LoginRequest>,
) -> Result<impl Responder, SessionError> {
    let request = request.0;
    let session = session_provider
        .login(request.username, request.password)
        .await?;

    let session_cookie = Cookie::build("sessionId", session.id)
        .path("/")
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(10)))
        .finish();
    Ok(HttpResponse::Ok().cookie(session_cookie).finish())
}

impl<T: ObjectId + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>
    FromRequest for SessionRes<T>
{
    type Error = SessionError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let Some(session_id) = req.cookie("sessionId") else {
                return Err(SessionError::InvalidOrMissingSession);
            };

            let Some(session_provider) = req.app_data::<Data<SessionProvider<T>>>() else {
                return Err(SessionError::InternalServerError);
            };

            let res = session_provider.validate(session_id.value().into()).await?;

            Ok(SessionRes { inner: res })
        })
    }
}

#[async_trait]
pub trait SessionBackend<T: ObjectId + Serialize + for<'de> Deserialize<'de>>: Send + Sync {
    async fn validate(&self, session_id: String) -> Result<T, SessionError>;
    async fn login(&self, username: String, password: String) -> Result<Session<T>, SessionError>;
}
