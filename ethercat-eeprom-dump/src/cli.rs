use clap::{Command, arg};

pub fn cli() -> Command {
    Command::new("ethercat-eeprom-dump")
        .subcommand_required(true)
        // add arg for interface
        .arg(
            arg!(--interface <INTERFACE> "EtherCAT interface to use")
                .required(true)
                .short('i')
                .help("EtherCAT interface to use"),
        )
        .subcommand(Command::new("ls").about("List all EtherCAT devices"))
        .subcommand(
            Command::new("dump")
                .about("Read the EEPROM of a device")
                .arg(
                    arg!(<SUBDEVICE> "Subdevice index to read from")
                        .value_parser(clap::value_parser!(usize))
                        .help("Subdevice index to read from"),
                )
                .arg(
                    arg!(--file <FILE> "File to save the EEPROM to")
                        .required(false)
                        .short('f')
                        .help("File to save the EEPROM to"),
                ),
        )
        .subcommand(
            Command::new("restore")
                .about("Write the EEPROM of a device")
                .arg(
                    arg!(<SUBDEVICE> "Subdevice index to write to")
                        .value_parser(clap::value_parser!(usize))
                        .help("Subdevice index to write to"),
                )
                .arg(
                    arg!(--file <FILE> "File to read the EEPROM from")
                        .required(true)
                        .short('f')
                        .help("File to read the EEPROM from"),
                ),
        )
        .subcommand(
            Command::new("read").about("Read a dumped EEPROM file").arg(
                arg!(--file <FILE> "File to parse")
                    .required(true)
                    .short('f')
                    .value_parser(clap::value_parser!(String))
                    .help("File to parse"),
            ),
        )
}
