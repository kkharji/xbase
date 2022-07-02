use crate::broadcast::Broadcast;
use crate::project::project;
use crate::store::*;
use crate::watch::WatchService;
use crate::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use xbase_proto::PathExt;
use xbase_proto::OK;

/// Handle RegisterRequest
pub async fn handle(root: PathBuf) -> Result<PathBuf> {
    log::info!("Register {}", root.as_path().name().unwrap());

    let (broadcast, logger_path) = if let Ok(broadcast) = root.try_get_broadcast().await {
        (broadcast.clone(), broadcast.address().clone())
    } else {
        let broadcast = Broadcast::new(&root).await.map(Arc::new)?;
        let address = broadcast.address().clone();
        broadcasters().await.insert(root.clone(), broadcast.clone());
        (broadcast, address)
    };

    tokio::spawn(async move {
        let mut projects = projects().await;

        if let Some(project) = projects.get(&root).map(Arc::clone) {
            let mut project = project.lock_owned().await;
            project.inc_clients();
            // NOTE: this doesn't make sense!
            project.ensure_server_support(None, &broadcast).await?;
            return Ok(());
        }
        let project = project(&root, &broadcast).await?;
        let name = project.name().to_string();
        let root = project.root().clone();
        let ignore = project.watchignore().clone();

        let project = Arc::new(Mutex::new(project));
        let handler = WatchService::new(
            &root,
            ignore,
            Arc::downgrade(&broadcast),
            Arc::downgrade(&project),
        )
        .await?;

        log::info!("[{}] Project added", name);

        projects.insert(root.clone(), project.clone());
        watchers()
            .await
            .insert(root.clone(), Arc::new(Mutex::new(handler)));
        project
            .lock()
            .await
            .ensure_server_support(None, &broadcast)
            .await?;
        OK(())
    });

    Ok(logger_path)
}
