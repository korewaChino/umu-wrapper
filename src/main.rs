use clap::Parser;
use color_eyre::Result;
use tracing_subscriber::EnvFilter;

mod cli;
mod config;
mod error;



fn main() -> Result<()> {
    // let config = config::Config::load(&config_path()?)?;

    // println!("{:#?}", config);

    // default level is `info`
    let default_envfilter = EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env()?;
    tracing_subscriber::fmt::fmt()
        .with_env_filter(default_envfilter)
        .init();

    let args = cli::Args::parse();
    args.run();

    Ok(())
}
