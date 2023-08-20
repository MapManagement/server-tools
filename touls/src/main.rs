#![allow(unused)]

use borgflux::run_borgflux;
use clap::{arg, command, ArgMatches, Command, Parser};

#[derive(Parser)]
struct ToulsCli {
    tool: String,
}

fn main() {
    let command_matches = command!()
        .subcommand(Command::new("borgflux").about("BorgBackup and InfluxDB combined"))
        .get_matches();
}

fn run_sub_tool(matches: ArgMatches) {
    match matches.subcommand() {
        Some(("borgflux", sub_matches)) => run_borgflux(),
        _ => unreachable!("Error!"),
    }
}
