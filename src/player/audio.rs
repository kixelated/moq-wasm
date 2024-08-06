use super::{CatalogError, Session};

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("catalog error: {0}")]
    Catalog(#[from] CatalogError),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::Error),

    #[error("missing namespace")]
    MissingNamespace,
}

pub struct Audio {
    info: moq_warp::catalog::Track,
    reader: moq_transfork::TrackReader,
}

impl Audio {
    pub async fn fetch(
        session: &mut Session,
        info: moq_warp::catalog::Track,
    ) -> Result<Self, AudioError> {
        tracing::info!("fetching audio track: {:?}", info);
        let broadcast = info
            .namespace
            .as_ref()
            .ok_or(AudioError::MissingNamespace)?;
        let broadcast = session.namespace(broadcast)?;

        let track = moq_transfork::Track::build(&info.name, 2).into();
        let reader = broadcast.subscribe(track).await?;

        Ok(Self { info, reader })
    }

    // NOTE: Can be called multiple times
    pub async fn run(&mut self) -> Result<(), AudioError> {
        while let Some(mut group) = self.reader.next_group().await? {
            while let Some(frame) = group.read_frame().await? {
                tracing::info!("audio frame: {:?}", frame.len());
            }
        }

        Ok(())
    }
}
