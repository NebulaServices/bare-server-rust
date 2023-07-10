use std::fmt::format;

use miette::{Diagnostic, Report as StackReport};
use reqwest::StatusCode;
use salvo::{
    async_trait, Depot, Request, Response, Writer,
    __private::tracing::{self},
    writer::Json,
};
use serde_json::{json, Value};
use thiserror::Error;
pub type Result<T> = core::result::Result<T, ErrorWithContext>;

pub struct ErrorWithContext {
    error: Error,
    context: String,
}

impl ErrorWithContext {
    pub fn new<T: Into<String>>(error: Error, context: T) -> Self {
        Self {
            error,
            context: context.into(),
        }
    }

    pub fn to_report(&self) -> miette::Report {
        miette::Report::new(self.error.to_owned()).wrap_err(self.context.to_owned())
    }
}

#[async_trait]
impl Writer for ErrorWithContext {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self.error {
            Error::Unknown
            | Error::HostNotFound
            | Error::ConnectionReset
            | Error::ConnectionRefused
            | Error::Generic(_)
            | Error::ConnectionTimeout => res.status_code(StatusCode::INTERNAL_SERVER_ERROR),
            Error::MissingBareHeader(_)
            | Error::InvalidBareHeader(_)
            | Error::UnknownBareHeader(_)
            | Error::InvalidHeader(_) => res.status_code(StatusCode::BAD_REQUEST),
            Error::ForbiddenBareHeader(_) => res.status_code(StatusCode::FORBIDDEN),
        };
        let report = self.to_report();
        tracing::error!("\n {report:?}");
        res.render(Json(self.error.to_json()));
    }
}

#[derive(Debug, Diagnostic, Error, Clone)]
#[error("oops!")]
pub enum Error {
    #[error("The Bare Server could not identify the cause of the issue")]
    #[diagnostic(code(UNKNOWN))]
    Unknown,
    #[error("The request did not include the required header {0}")]
    #[diagnostic(code(MISSING_BARE_HEADER))]
    MissingBareHeader(String),
    #[error("Received an unrecognizable header value: {0}")]
    #[diagnostic(code(INVALID_BARE_HEADER))]
    InvalidBareHeader(String),
    #[error("Received a forbidden header value: {0}")]
    #[diagnostic(code(FORBIDDEN_BARE_HEADER))]
    ForbiddenBareHeader(String),
    // NOTE: This is unused, checking for unknown headers is a waste of compute.
    // I may gate this behind a feature flag at a later date.
    #[error("Received unknown bare header {0}")]
    #[diagnostic(code(UNKNOWN_BARE_HEADER))]
    UnknownBareHeader(String),
    // Why does this exist? This is a duplicate of InvalidBareHeader...
    #[error("Received a blacklisted header value: {0}")]
    #[diagnostic(code(INVALID_HEADER))]
    InvalidHeader(String),
    #[error("The DNS lookup for the host failed.")]
    #[diagnostic(code(HOST_NOT_FOUND))]
    HostNotFound,
    #[error("The connection to the remote was closed early.")]
    #[diagnostic(code(CONNECTION_RESET))]
    ConnectionReset,
    #[error("The connection to the remote was refused.")]
    #[diagnostic(code(CONNECTION_REFUSED))]
    ConnectionRefused,
    #[error("The remote didn't respond with headers/body in time.")]
    #[diagnostic(code(CONNECTION_TIMEOUT))]
    ConnectionTimeout,
    #[error("{0}")]
    #[diagnostic(code(UNKNOWN))]
    Generic(String),
}

impl Error {
    pub fn to_json(&self) -> Value {
        let id: String = match self {
            Error::Unknown => "unknown".into(),
            Error::MissingBareHeader(header) | Error::InvalidBareHeader(header) => {
                format!("request.headers.{}", header.to_lowercase())
            }
            Error::ForbiddenBareHeader(header) => format!("error.temp.forbidden_header.{header}"),
            Error::UnknownBareHeader(header) => format!("error.temp.unknown_bare_header.{header}"),
            Error::InvalidHeader(header) => format!("error.temp.invalid_header.{header}"),
            Error::HostNotFound => format!("error.http.not_found"),
            Error::ConnectionReset => format!("error.http.reset"),
            Error::ConnectionRefused => format!("error.http.refused"),
            Error::ConnectionTimeout => format!("error.http.timeout"),
            Error::Generic(kind) => format!("request.tomphttp-rs.{kind}"),
        };

        json!({
            "code": format!("{}", self.code().expect("This should always be defined.")),
            "id": id,
            "message": format!("{self}")

        })
    }
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let report: StackReport = self.into();
        tracing::error!("\n {report:?}");
        res.render(format!("{report:?}"));
    }
}
