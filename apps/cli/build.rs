use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PROFILE")? == "release" {
        build_web_ui()?;
    }
    Ok(())
}

fn build_web_ui() -> Result<(), Box<dyn std::error::Error>> {
    let webui_dir = Path::new("../../apps/webui");

    if !webui_dir.exists() {
        return Err("Web UI directory not found".into());
    }

    let status = Command::new("bun")
        .current_dir(webui_dir)
        .args(["run", "build"])
        .status()?;

    if !status.success() {
        return Err("Failed to build web UI".into());
    }

    Ok(())
}
