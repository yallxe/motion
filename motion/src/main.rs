use anyhow::Result;
use config::Configuration;

pub mod config;
pub mod proxy;

#[macro_use] extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        println!("No RUST_LOG set. Setting to 'info' by default");
        std::env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::try_init().unwrap();
    
    let cmd = clap::Command::new(clap::crate_name!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to the configuration file")
                .default_value("motion.yml"),
        )
        .get_matches();

    let config_path = cmd.get_one::<String>("config").unwrap();

    let config = match Configuration::from_file(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config from '{}': {}", config_path, e);
            return Err(e.into());
        }
    };
    
    info!("Loaded config from {}", config_path);

    let proxy = proxy::server::ProxyServer::new(config);
    Ok(proxy.run().await?)
}
