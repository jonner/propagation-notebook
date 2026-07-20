use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] toasty::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let code = match &self {
            Error::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, self.to_string()).into_response()
    }
}
