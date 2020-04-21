use clap::{App, Arg};
use jenkins_api::JenkinsBuilder;
use serde::Deserialize;


#[derive(Deserialize, Debug)]
struct Jobs {
    jobs: Vec<Builds>
}
#[derive(Deserialize, Debug)]
struct Builds {
    builds: Vec<Build>
}
#[derive(Deserialize, Debug)]
struct Build {
    number: i32,
    url: String,
}

fn main() {
    let matches = App::new("jenky")
        .version("0.0.1")
        .author("Gavin Rossiter <rossiter.gavin@gmail.com>")
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

    let jenkins = JenkinsBuilder::new("http://localhost:5001").build().unwrap(); 
    // match jenkins.get_view("All"){ 
    //     Ok(home) => println!("{}", home.jobs),
    //     Err(e)=> eprintln!("{}",e)
    // }
    for job in jenkins.get_view("All").unwrap().jobs {
        let path =jenkins_api::client::Path::Job {
            name: "apple.cmats",
            configuration: None,
        };
        use jenkins_api::client::TreeBuilder;

        let tree = TreeBuilder::object("jobs").with_subfield(TreeBuilder::object("builds").with_subfield("url").with_subfield("number")).build();
        let builds : Jobs = jenkins.get_object_as(path,tree).unwrap();
        dbg!(builds);
        // println!("{}", job.get_full_job(jenkins_client).unwrap().last_build.unwrap().display_name.unwrap());
    }



    // // You can check the value provided by positional arguments, or option arguments
    // if let Some(o) = matches.value_of("output") {
    //     println!("Value for output: {}", o);
    // }

    // if let Some(c) = matches.value_of("config") {
    //     println!("Value for config: {}", c);
    // }

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match matches.occurrences_of("debug") {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     3 | _ => println!("Don't be crazy"),
    // }

    // // You can check for the existence of subcommands, and if found use their
    // // matches just as you would the top level app
    // if let Some(ref matches) = matches.subcommand_matches("test") {
    //     // "$ myapp test" was run
    //     if matches.is_present("list") {
    //         // "$ myapp test -l" was run
    //         println!("Printing testing lists...");
    //     } else {
    //         println!("Not printing testing lists...");
    //     }
    // }

    // Continued program logic goes here...
}
