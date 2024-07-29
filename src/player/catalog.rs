#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("missing catalog")]
    Missing,

    #[error("encoding error: {0}")]
    Parse(#[from] moq_catalog::Error),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::MoqError),
}

pub struct Catalog {
    pub broadcast: moq_transfork::Broadcast,
    pub parsed: moq_catalog::Root,
}

impl Catalog {
    pub async fn fetch(
        subscriber: &mut moq_transfork::Subscriber,
        broadcast: moq_transfork::Broadcast,
    ) -> Result<Self, CatalogError> {
        let track = moq_transfork::Track::new(".catalog", 0).build();
        let mut track = subscriber.subscribe(broadcast.clone(), track).await?;

        let mut group = track.next_group().await?.ok_or(CatalogError::Missing)?;
        let object = group.read_frame().await?.ok_or(CatalogError::Missing)?;

        let parsed = moq_catalog::Root::from_slice(&object)?;

        Ok(Self { broadcast, parsed })
    }

    pub fn video(&self) -> impl Iterator<Item = &moq_catalog::Track> {
        self.parsed
            .tracks
            .iter()
            .filter(|track| track.selection_params.width.is_some())
    }

    pub fn audio(&self) -> impl Iterator<Item = &moq_catalog::Track> {
        self.parsed
            .tracks
            .iter()
            .filter(|track| track.selection_params.samplerate.is_some())
    }
}
