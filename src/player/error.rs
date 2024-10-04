#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::Error),

    #[error("webtransport error: {0}")]
    WebTransport(#[from] web_transport::wasm::Error),

    #[error("catalog error: {0}")]
    Catalog(#[from] moq_warp::catalog::Error),

    #[error("webcodecs error: {0}")]
    WebCodecs(#[from] web_codecs::Error),

    #[error("media error: {0}")]
    Warp(#[from] moq_warp::media::Error),

    #[error("invalid url")]
    InvalidUrl,

    #[error("invalid fingerprint")]
    InvalidFingerprint,

    #[error("http error: {0}")]
    Http(#[from] gloo_net::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
