use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Blazingly fast launcher for chameleon shell")]
pub struct Command {
    #[argh(switch, short = 't', description = "toggle launcher visibility")]
    pub toggle: bool,
}
