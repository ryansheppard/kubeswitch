use anyhow::Result;
use clap::Parser;
use kube::config::Kubeconfig;
use std::fs::File;

mod cli;
mod config;
mod kubernetes;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let kubeconfig_path = config::get_kubeconfig_path()?;
    let config = Kubeconfig::read_from(&kubeconfig_path)?;

    let config = match args.action {
        cli::Action::Context => config::select_context(config, &args.item_name).await?,
        cli::Action::Namespace => config::select_namespace(config, &args.item_name).await?,
    };

    let new_file = File::create(&kubeconfig_path)?;
    serde_yaml::to_writer(new_file, &config)?;

    Ok(())
}
