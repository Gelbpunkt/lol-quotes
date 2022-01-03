#[derive(Debug)]
pub enum Error {
    Http(hyper::http::Error),
    Hyper(hyper::Error),
    Json(simd_json::Error),
    TwilightHttp(twilight_http::Error),
    Deserialization(twilight_http::response::DeserializeBodyError),
}

impl From<hyper::http::Error> for Error {
    fn from(err: hyper::http::Error) -> Self {
        Self::Http(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::Hyper(err)
    }
}

impl From<simd_json::Error> for Error {
    fn from(err: simd_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<twilight_http::Error> for Error {
    fn from(err: twilight_http::Error) -> Self {
        Self::TwilightHttp(err)
    }
}

impl From<twilight_http::response::DeserializeBodyError> for Error {
    fn from(err: twilight_http::response::DeserializeBodyError) -> Self {
        Self::Deserialization(err)
    }
}
