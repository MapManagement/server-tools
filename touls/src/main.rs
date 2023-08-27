#![allow(unused)]

use borgflux::run_borgflux;
use clap::{arg, command, Arg, ArgMatches, Command, Parser};
use wakey::wake_on_lan;

#[derive(Parser)]
struct ToulsCli {
    tool: String,
}

fn main() {
    let command_matches = command!()
        .subcommand_required(true)
        .propagate_version(true)
        .subcommand(Command::new("borgflux").about("BorgBackup and InfluxDB combined"))
        .subcommand(
            Command::new("wake_on_lan")
                .about("Send a magic packet to to MAC address to start a target machine")
                .arg(Arg::new("mac_address")),
        )
        .get_matches();

    run_sub_tool(command_matches);
}

fn run_sub_tool(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("borgflux", sub_matches)) => run_borgflux(),
        Some(("wake_on_lan", sub_matches)) => {
            wake_on_lan(sub_matches.get_one::<String>("mac_address").unwrap());
        }
        _ => unreachable!("Error!"),
    }
}
