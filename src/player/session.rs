use url::Url;

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("webtransport error: {0}")]
    WebTransport(#[from] web_transport::SessionError),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::MoqError),

    #[error("url error: invalid scheme")]
    InvalidScheme,

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error("fingerprint error: {0}")]
    Fingerprint(#[from] FingerprintError),
}

#[derive(thiserror::Error, Debug)]
pub enum FingerprintError {
    #[error("http error: {0}")]
    Http(#[from] gloo_net::Error),

    #[error("hex error: {0}")]
    Hex(#[from] hex::FromHexError),
}

pub struct Session {
    pub url: Url,
    pub subscriber: moq_transfork::Subscriber,
}

impl Session {
    pub async fn connect(url: &str) -> Result<Self, SessionError> {
        let url = Url::parse(url)?;
        if url.scheme() != "https" {
            return Err(SessionError::InvalidScheme);
        }

        let session = web_transport_wasm::Session::new(url.as_str())
            .allow_pooling(false)
            .congestion_control(web_transport_wasm::CongestionControl::LowLatency)
            .require_unreliable(true);

        // TODO Unfortunately, WebTransport doesn't work correctly with self-signed certificates.
        // Until that gets fixed, we need to perform a HTTP request to fetch the certificate hashes.
        let session = match url.host_str() {
            Some("localhost") => {
                let fingerprint = Self::fingerprint(&url).await?;
                session.server_certificate_hashes(vec![fingerprint])
            }
            _ => session,
        };

        let session = session.connect().await?;

        let subscriber = moq_transfork::Client::new(session.into())
            .subscriber()
            .await?;

        Ok(Self { url, subscriber })
    }

    async fn fingerprint(url: &Url) -> Result<Vec<u8>, FingerprintError> {
        let mut fingerprint = url.clone();
        fingerprint.set_path("fingerprint");

        let resp = gloo_net::http::Request::get(fingerprint.as_str())
            .send()
            .await?;

        let body = resp.text().await?;
        let fingerprint = hex::decode(body.trim())?;

        Ok(fingerprint)
    }
}
