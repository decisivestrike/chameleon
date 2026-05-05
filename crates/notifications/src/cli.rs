use argh::FromArgs;
use chameleon_shared::utils::resolve_path;
use std::path::PathBuf;

#[derive(FromArgs)]
#[argh(description = "Notifications daemon for Chameleon")]
pub struct Args {
    #[argh(
        option,
        short = 'c',
        description = "config path",
        from_str_fn(resolve_path)
    )]
    pub config_path: Option<PathBuf>,

    #[argh(
        option,
        short = 's',
        description = "styles path",
        from_str_fn(resolve_path)
    )]
    pub styles_path: Option<PathBuf>,
}
