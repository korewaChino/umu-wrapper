use clap::Parser;
use tracing::{error, info};

pub fn config_path() -> String {
    let config_path = dirs::config_dir()
        .expect("Failed to get config directory")
        .join("umu-wrapper.toml")
        .to_string_lossy()
        .to_string();
    config_path
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short = 'c', env = "UMUWRAPPER_CONFIG_PATH", default_value_t = config_path())]
    pub config: String,

    #[clap(short = 'p', long)]
    pub profile: String,
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,
}

#[derive(Parser, Debug)]
pub enum Subcommand {
    #[clap(name = "run")]
    Run { args: Vec<String> },
}

impl Args {
    pub fn run(&self) {
        let config = crate::config::Config::load(&self.config).expect("Failed to load config");
        match &self.subcommand {
            Some(Subcommand::Run { args }) => {
                info!("Attempting to resolve profile: {}", self.profile);

                let mut profile = config
                    .resolve_profile(&self.profile)
                    .expect("Failed to resolve profile");

                if !args.is_empty() {
                    profile.exe = args[0].clone().into();
                    profile.args = Some(args[1..].to_vec());
                }

                // info!("{:#?}", profile);

                info!("Starting UMU with profile: {}", profile.name);

                let _ = profile.run_profile();
            }
            None => {
                error!("No subcommand provided");
            }
        }
    }
}
