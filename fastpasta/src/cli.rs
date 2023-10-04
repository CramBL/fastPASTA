use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("check")
        .about("Do checks on raw data")
        .subcommand(
            Command::new("all").about("Running and sanity checks").arg(
                Arg::new("SYSTEM")
                    .action(ArgAction::Set)
                    .help("the system to check against, such as ITS or ITS-Stave"),
            ),
        )
}
