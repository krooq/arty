use clap::{App, Arg};

fn main() {
    let matches = App::new("jenky")
        .version("0.0.1")
        .author("Gavin R. <rossiter.gavin@gmail.com>")
        .about("Allows you download the artifacts from any jenkins build!")
        .arg(
            Arg::with_name("ip")
                .help("The IP address of your jenkins server")
                .index(1),
        )
        // .arg(
        //     Arg::with_name("extract")
        //         .short('x')
        //         .long("extract")
        //         .help("Extracts the artifacts after downloading"),
        // )
        // .arg(
        //     Arg::index("dir")
        //         .help("Sets the download directory")
        //         .index(1)
        //         .default_value("."),
        // )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(o) = matches.value_of("output") {
        println!("Value for output: {}", o);
    }

    if let Some(c) = matches.value_of("config") {
        println!("Value for config: {}", c);
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches.occurrences_of("debug") {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        3 | _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(ref matches) = matches.subcommand_matches("test") {
        // "$ myapp test" was run
        if matches.is_present("list") {
            // "$ myapp test -l" was run
            println!("Printing testing lists...");
        } else {
            println!("Not printing testing lists...");
        }
    }

    // Continued program logic goes here...
}
