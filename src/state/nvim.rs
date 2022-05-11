use super::DaemonState;
use crate::{daemon::Workspace, nvim::Nvim};
use anyhow::Result;

pub fn set_watch_script(root: &str, target: &str, is_watching: bool) -> String {
    format!(
        r#"
        if not vim.g.xcodebase.watch['{root}'] then 
            vim.g.xcodebase.watch['{root}'] = vim.empty_dict() 
        end

        vim.g.xcodebase.watch['{target}'] = {is_watching}
        "#
    )
}

pub fn set_state_script(root: &str, project: &str) -> String {
    format!(
        r#"
        vim.g.xcodebase.projects['{root}'] = vim.json.decode('{project}')
        "#
    )
}

/// Update project state in workspace
pub async fn sync_project_state_ws(ws: &Workspace) -> Result<()> {
    let root = ws.root.to_string_lossy();
    let script = format!(
        "let g:xcodebase.projects['{}'] = v:lua.vim.json.decode('{}')",
        root,
        ws.project.to_string()?
    );

    for (pid, nvim) in ws.clients.iter() {
        nvim.exec(&script, false).await?;

        tracing::info!("{pid}: synced project state");
    }

    Ok(())
}

/// Update watching state in workspace
pub async fn sync_is_watching_ws(ws: &Workspace) -> Result<()> {
    let root = ws.root.to_string_lossy();

    for (target, _) in ws.project.targets() {
        // let is_watching = ws.is_watching(target);
        let is_watching = false;
        let set_script = format!("let g:xcodebase.watch['{}'] = {{}}", root,);
        let watch_script =
            format!("let g:xcodebase.watch['{root}']['{target}'] = v:{is_watching}",);

        for (pid, nvim) in ws.clients.iter() {
            nvim.exec(&set_script, false).await?;
            nvim.exec(&watch_script, false).await?;

            tracing::info!("{pid} synced is_watching state");
        }
    }

    Ok(())
}

/// Set all client nvim state's is watching.
pub async fn sync_is_watching_all(state: DaemonState) -> Result<()> {
    for (_, ws) in state.lock().await.workspaces.iter() {
        sync_is_watching_ws(ws).await?
    }

    Ok(())
}

/// Update project state in all
pub async fn sync_project_state_all(state: DaemonState) -> Result<()> {
    for (_, ws) in state.lock().await.workspaces.iter() {
        sync_project_state_ws(ws).await?;
    }

    Ok(())
}

//         {
//             let root = self.root.to_string_lossy();
//             let mut setup_state = format!("let g:xcodebase['{root}'] = {{");

//             for (target, _) in self.targets() {
//                 setup_state.push_str(&format!("'{target}': {{"));
//                 // self.is_watching(&self, target: &str)
//                 let is_watching = false;
//                 setup_state.push_str(&format!("'is_watching': v:{is_watching}"));
//                 setup_state.push_str(&format!("}}"));
//             }

//             setup_state.push_str(&format!("}}"));

//             nvim.exec(&setup_state, false).await?;
//         }
