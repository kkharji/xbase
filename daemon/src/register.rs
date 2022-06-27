use crate::compile;
use crate::constants::DAEMON_STATE;
use crate::logger::Logger;
use crate::util::log_request;
use crate::Result;
use xbase_proto::LoggingTask;
use xbase_proto::RegisterRequest;

/// Handle RegisterRequest
pub async fn handle(RegisterRequest { client }: RegisterRequest) -> Result<Vec<LoggingTask>> {
    log_request!("Register", client);
    let root = client.root.as_path();
    let logger = Logger::new(
        format!("Register {}", root.display()),
        format!("register_{}.log", client.abbrev_root().replace("/", "_")),
        root,
    )
    .await?;

    let state = DAEMON_STATE.clone();
    let ref mut state = state.lock().await;

    let mut tasks = state
        .loggers
        .get_by_project_root(&client.root)
        .iter()
        .map(|l| l.to_logging_task())
        .collect::<Vec<_>>();

    tasks.push(logger.to_logging_task());

    let weak_logger = state.loggers.push(logger);
    let logger = weak_logger.upgrade().unwrap();

    if let Ok(project) = state.projects.get_mut(&client.root) {
        project.add_client(client.pid);
    } else {
        state.projects.add(&client).await?;
        let project = state.projects.get(&client.root).unwrap();
        let watchignore = project.watchignore().clone();
        let name = project.name().to_string();

        state.watcher.add(&client, watchignore, &name).await?;
    }

    if compile::ensure_server_support(state, &client, None, &logger).await? {
        logger.append("setup: ✅");
    }

    Ok(tasks)
}
