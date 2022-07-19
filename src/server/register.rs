use super::*;
use crate::runtime::ProjectRuntime;
use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

/// Register a project root
#[derive(Debug, Serialize, Deserialize, TypeDef)]
pub struct RegisterRequest {
    pub id: u32,
    pub root: PathBuf,
}

#[async_trait]
impl RequestHandler<PathBuf> for RegisterRequest {
    async fn handle(self) -> Result<PathBuf> {
        let RegisterRequest { id, root } = self;
        let mut runtimes = runtimes().await;
        tracing::trace!("{:#?}", runtimes);

        if let Some(runtime) = runtimes.get_mut(&root) {
            if runtime.contains(&id) {
                return Err(Error::Unexpected(
                    "Trying to adding a connected client!".into(),
                ));
            }

            let address = runtime.broadcaster_adderss().clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                runtimes.get_mut(&root).unwrap().connect(id);
            });

            return Ok(address);
        }

        let (rloop, mut runtime) = match ProjectRuntime::new(root.clone()).await {
            Ok(v) => v,
            Err(err) => {
                let name = root.as_path().name().unwrap();
                return Err(Error::Setup(name, err.to_string()));
            }
        };

        let address = runtime.broadcaster_adderss().clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            runtime.insert(id);
            runtimes.insert(root, runtime);
            drop(runtimes);
            rloop.start(id).await;
        });

        Ok(address)
    }
}
