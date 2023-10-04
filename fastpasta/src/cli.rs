use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    let mut fastpasta = Command::new("fastpasta")
        .about("Do checks or view raw data")
        .subcommand(
            Command::new("check")
                .about("Do checks on raw data")
                .arg(
                    Arg::new("filter-link")
                        .action(ArgAction::Set)
                        .help("Filter by a GBT link"),
                )
                .subcommand_required(true)
                .subcommand(
                    Command::new("sanity")
                        .about("Sanity checks are stateless")
                        .subcommand(Command::new("its")),
                )
                .subcommand(
                    Command::new("all")
                        .about("All possible checks")
                        .subcommand(Command::new("its").about("All checks applicable to ITS"))
                        .subcommand(
                            Command::new("its-stave").about("All checks applicable to ITS Staves"),
                        ),
                ),
        );

    fastpasta.subcommand(
        Command::new("view")
            .about("View raw data")
            .subcommand(Command::new("rdh").about("View RDHS"))
            .subcommand(Command::new("its-readout-frames").about("View ITS readout frames"))
            .subcommand(
                Command::new("its-readout-frames-data")
                    .about("View ITS readout frames with Data words"),
            ),
    )
}
