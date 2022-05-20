use super::*;

impl Runner {
    pub async fn run_as_macos_app(self, settings: BuildSettings) -> Result<JoinHandle<Result<()>>> {
        let nvim = self.state.clients.get(&self.client.pid)?;
        let ref mut logger = nvim.new_logger("Run", &self.target, &self.direction);

        logger.log_title().await?;
        logger.open_win().await?;

        tokio::spawn(async move {
            let program = settings.path_to_output_binary()?;
            let mut stream = runner::run(&program).await?;

            tracing::debug!("Running binary {program:?}");

            use xcodebuild::runner::ProcessUpdate::*;
            // NOTE: This is required so when neovim exist this should also exit
            while let Some(update) = stream.next().await {
                let state = DAEMON_STATE.clone();
                let state = state.lock().await;
                let nvim = state.clients.get(&self.client.pid)?;
                let mut logger = nvim.new_logger("Run", &self.target, &self.direction);

                // NOTE: NSLog get directed to error by default which is odd
                match update {
                    Stdout(msg) => {
                        logger.log(msg).await?;
                    }
                    Error(msg) | Stderr(msg) => {
                        logger.log(format!("[Error]  {msg}")).await?;
                    }
                    Exit(ref code) => {
                        let success = code == "0";
                        let msg = string_as_section(if success {
                            "".into()
                        } else {
                            format!("Panic {code}")
                        });

                        logger.log(msg).await?;
                        logger.set_status_end(success, true).await?;
                    }
                }
            }
            Ok(())
        })
        .pipe(Ok)
    }
}
