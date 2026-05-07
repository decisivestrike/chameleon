use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Blazingly fast launcher for Chameleon")]
pub struct Args {
    #[argh(switch, short = 't', description = "toggle launcher visibility")]
    pub toggle: bool,
}
