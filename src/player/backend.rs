use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use tokio::sync::watch;
use url::Url;

use super::{Audio, Config, Error, Result, Video};

#[derive(Default)]
pub struct Backend {}

impl Backend {
    async fn connect(url: &str) -> Result<moq_transfork::Subscriber> {
        let url = Url::parse(url).map_err(|_| Error::InvalidUrl)?;
        if url.scheme() != "https" {
            return Err(Error::InvalidUrl);
        }

        let session = web_transport_wasm::Session::build(url.clone())
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

        Ok(moq_transfork::Client::new(session.into())
            .subscriber()
            .await?)
    }

    async fn fingerprint(url: &Url) -> Result<Vec<u8>> {
        let mut fingerprint = url.clone();
        fingerprint.set_path("fingerprint");

        let resp = gloo_net::http::Request::get(fingerprint.as_str())
            .send()
            .await?;

        let body = resp.text().await?;
        let fingerprint = hex::decode(body.trim()).map_err(|_| Error::InvalidFingerprint)?;

        Ok(fingerprint)
    }

    async fn run(&mut self, config: Config) -> Result<()> {
        let broadcast = match config.attrs.broadcast.as_ref() {
            Some(broadcast) => moq_transfork::Broadcast::new(broadcast),
            None => return Ok(()),
        };

        let session = match config.attrs.src.as_ref() {
            Some(url) => Self::connect(url).await?,
            None => return Ok(()),
        };

        // Fetch the catalog
        let broadcast = session.namespace(broadcast)?;
        let broadcast = moq_warp::media::BroadcastConsumer::load(broadcast).await?;

        tracing::info!(catalog = ?broadcast.catalog(), "starting playback");

        let mut tasks = FuturesUnordered::new();

        if let Some(canvas) = config.canvas {
            let video = Video::new(broadcast.clone(), canvas);
            tasks.push(async move { video.run().await }.boxed_local());
        }

        let audio = Audio::new(broadcast);
        tasks.push(async move { audio.run().await }.boxed_local());

        loop {
            tokio::select! {
                Some(res) = tasks.next() => res?,
                else => return Ok(()),
            }
        }
    }

    #[tracing::instrument("backend", skip_all)]
    pub async fn watch(&mut self, mut config: watch::Receiver<Config>) -> Result<()> {
        while let Ok(_) = config.changed().await {
            let current = config.borrow_and_update().clone();
            tracing::info!(config = ?current);

            tokio::select! {
                Ok(()) = config.changed() => continue,
                res = self.run(current) => {
                    if let Err(err) = res {
                        tracing::error!(?err, "backend error");
                    }
                },
            };
        }

        Ok(())
    }
}
