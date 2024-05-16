//! This Rust program is a command-line utility for updating the A record of a domain on Cloudflare
//! to match the current outside IP address.
use anyhow::bail;
use clap::{command, crate_description, crate_version, Arg, ArgAction, ArgMatches};
use reqwest::blocking::Client as RqClient;

use crate::config::Config;
use crate::network::get_outside_ip;

mod cloudflare;
mod config;
mod network;

fn main() {
    match app() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn app() -> anyhow::Result<()> {
    env_logger::init();

    let arg_matches = parse_args();
    let api_key = arg_matches.get_one::<String>("api_key").unwrap();
    let zone_id = arg_matches.get_one::<String>("zone_id").unwrap();
    let domain = arg_matches.get_one::<String>("domain").unwrap();
    let dry_run = arg_matches.get_flag("dry_run");

    if dry_run {
        log::debug!("Performing dry run");
    }

    let mut config = Config::default();
    if let Some(config_dir) = arg_matches.get_one::<String>("config_dir") {
        log::debug!("Setting config directory to: {config_dir}");
        config.save_dir = config_dir.into();
    }
    config.load()?;

    let client = RqClient::new();
    let outside_ip = match get_outside_ip(&client, None) {
        Ok(ip) => ip,
        Err(e) => {
            bail!("Error: {e}");
        }
    };

    if let Some(config_outside_ip) = config.outside_ip {
        if outside_ip == config_outside_ip {
            log::info!("Outside IP has not changed. Nothing to do.");

            return Ok(());
        }
    }

    // Save the outside IP to the configuration, so we can exit early next time if it hasn't changed
    config.outside_ip = Some(outside_ip);
    if let Err(e) = config.save() {
        log::error!("Error: {e}");
    } else {
        log::info!("Config saved");
    }

    log::debug!("Processing domain: {}", domain);
    log::debug!("Outside IP: {}", outside_ip);

    let mut cloudflare_client = cloudflare::Handler::try_new(api_key, zone_id)?;

    // Get the A record
    let cloudflare_ip = cloudflare_client.get_a_record(domain)?;

    log::debug!("Cloudflare IP: {cloudflare_ip}");

    if outside_ip == cloudflare_ip {
        log::info!("Cloudflare IP is already up to date");
    } else {
        log::info!("Need to update Cloudflare IP");
        if dry_run {
            log::debug!("Dry run: Would update A record for {domain}: {outside_ip}");
        } else {
            cloudflare_client.update_a_record(outside_ip)?;
            log::info!("A record for {domain} updated with {outside_ip} at Cloudflare");
            config.cloudflare_ip = Some(outside_ip);

            if let Err(e) = config.save() {
                log::error!("Error: {e}");
            } else {
                log::info!("Config saved");
            }
        }
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    command!()
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::new("api_key")
                .short('k')
                .long("api-key")
                .required(true)
                .env("CDU_API_KEY")
                .help("Cloudflare API key"),
        )
        .arg(
            Arg::new("zone_id")
                .short('z')
                .long("zone-id")
                .required(true)
                .env("CDU_ZONE_ID")
                .help("Cloudflare zone ID"),
        )
        .arg(
            Arg::new("domain")
                .short('d')
                .long("domain")
                .required(true)
                .env("CDU_DOMAIN")
                .help("Domain name to update the A record of"),
        )
        .arg(
            Arg::new("dry_run")
                .short('n')
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .env("CDU_DRY_RUN")
                .help("Do not update the A record"),
        )
        .arg(
            Arg::new("config_dir")
                .short('c')
                .long("config-dir")
                .env("CDU_CONFIG_DIR")
                .help("Directory to save the configuration file in"),
        )
        .get_matches()
}
