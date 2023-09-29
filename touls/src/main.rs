#![allow(unused)]

use std::path::PathBuf;

use borgflux::run_borgflux;
use clap::{arg, command, Arg, ArgMatches, Command, Parser};
use wakey_wakey::wake_on_lan;

#[derive(Parser)]
struct ToulsCli {
    tool: String,
}

fn main() {
    let command_matches = command!()
        .subcommand_required(true)
        .propagate_version(true)
        .subcommand(
            Command::new("borgflux")
                .about("BorgBackup and InfluxDB combined")
                .arg(
                    Arg::new("config_file")
                        .short('c')
                        .long("config_file")
                        .value_parser(clap::builder::PathBufValueParser::new())
                        .help("Configuration file."),
                ),
        )
        .subcommand(
            Command::new("wake_on_lan")
                .about("Send a magic packet to to MAC address to start a target machine")
                .arg(
                    Arg::new("mac_address")
                        .short('m')
                        .long("mac_address")
                        .value_parser(clap::builder::NonEmptyStringValueParser::new())
                        .help("Target MAC address to sent magic packet to."),
                ),
        )
        .get_matches();

    run_sub_tool(command_matches);
}

fn run_sub_tool(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("borgflux", sub_matches)) => run_borgflux(
            sub_matches
                .get_one::<PathBuf>("config_file")
                .unwrap()
                .to_str()
                .unwrap(),
        ),
        Some(("wake_on_lan", sub_matches)) => {
            wake_on_lan(sub_matches.get_one::<String>("mac_address").unwrap());
        }
        _ => unreachable!("Error!"),
    }
}
