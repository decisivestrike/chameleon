use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "🦎 Modular and highly customizable Wayland shell")]
pub struct Args {
    #[argh(subcommand)]
    pub cmd: Option<CliCommand>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CliCommand {
    HealthCheck(HealthCheckCommand),
}

#[derive(FromArgs)]
#[argh(subcommand)]
#[argh(description = "Make healthcheck")]
#[argh(name = "health")]
pub struct HealthCheckCommand {}
