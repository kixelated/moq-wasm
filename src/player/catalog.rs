use super::Session;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("encoding error: {0}")]
    Parse(#[from] moq_warp::catalog::Error),

    #[error("transfork error: {0}")]
    Transfork(#[from] moq_transfork::Error),
}

pub struct Catalog {
    pub broadcast: moq_transfork::Broadcast,
    pub root: moq_warp::catalog::Root,
}

impl Catalog {
    pub async fn fetch(
        session: &mut Session,
        broadcast: moq_transfork::Broadcast,
    ) -> Result<Self, CatalogError> {
        let namespace = session.namespace(broadcast.clone())?;
        let root = moq_warp::catalog::Reader::subscribe(namespace)
            .await?
            .read()
            .await?;

        Ok(Self { broadcast, root })
    }

    pub fn video(&self) -> impl Iterator<Item = &moq_warp::catalog::Track> {
        self.root
            .tracks
            .iter()
            .filter(|track| track.selection_params.width.is_some())
    }

    pub fn audio(&self) -> impl Iterator<Item = &moq_warp::catalog::Track> {
        self.root
            .tracks
            .iter()
            .filter(|track| track.selection_params.samplerate.is_some())
    }
}
