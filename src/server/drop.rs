use super::*;
use crate::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Drop a given set of roots to be dropped (i.e. unregistered)
#[derive(Debug, Serialize, Deserialize, TypeDef)]
pub struct DropRequest {
    pub id: u32,
    pub roots: Vec<PathBuf>,
}

#[async_trait]
impl RequestHandler<()> for DropRequest {
    async fn handle(self) -> Result<()> {
        let DropRequest { roots, id } = self;
        let mut runtimes = runtimes().await;
        let mut drop_runtimes = vec![];

        for root in roots.into_iter() {
            if !runtimes.contains_key(&root) {
                continue;
            }
            let runtime = runtimes.get_mut(&root).unwrap();
            runtime.disconnect(id.clone());
            if runtime.is_closed() {
                drop_runtimes.push(root)
            }
        }
        for root in drop_runtimes {
            runtimes.remove(&root);
        }

        Ok(())
    }
}
