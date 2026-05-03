use resolve_path::PathResolveExt;
use std::path::PathBuf;

/// For argh
fn resolve_path(value: &str) -> Result<PathBuf, String> {
    Ok(value
        .try_resolve()
        .map_err(|e| e.to_string())?
        .to_path_buf())
}
