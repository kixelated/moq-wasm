use super::CatalogError;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("catalog error: {0}")]
    Catalog(#[from] CatalogError),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::MoqError),
}

pub struct Audio {
    info: moq_catalog::Track,
    reader: moq_transfork::TrackReader,
}

impl Audio {
    pub async fn fetch(
        subscriber: &mut moq_transfork::Subscriber,
        info: moq_catalog::Track,
    ) -> Result<Self, AudioError> {
        let broadcast =
            moq_transfork::Broadcast::new(info.namespace.as_ref().ok_or(CatalogError::Missing)?);
        let track = moq_transfork::Track::new(&info.name, 2).build();

        let reader = subscriber.subscribe(broadcast, track).await?;

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
