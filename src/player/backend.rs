use moq_transfork::util::State;

use super::Config;

pub struct Backend {
    config: State<Config>,
}

impl Backend {
    pub fn new(config: State<Config>) -> Self {
        Self { config }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let config = self.config.lock();

            tracing::info!(attrs = ?config.attrs, "update");

            match config.modified() {
                Some(notify) => notify.await,
                None => return Ok(()), // detached
            };
        }
    }
}
