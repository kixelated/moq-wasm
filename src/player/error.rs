#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::Error),

    #[error("webtransport error: {0}")]
    WebTransport(#[from] web_transport_wasm::Error),

    #[error("catalog error: {0}")]
    Catalog(#[from] moq_warp::catalog::Error),

    #[error("webcodecs error: {0}")]
    WebCodecs(#[from] web_codecs::Error),

    #[error("fmp4 error: {0}")]
    Fmp4(#[from] moq_warp::fmp4::Error),

    #[error("invalid url")]
    InvalidUrl,

    #[error("invalid fingerprint")]
    InvalidFingerprint,
}

pub type Result = std::result::Result<(), Error>;
