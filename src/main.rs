use jenkins_api::JenkinsBuilder;
use serde::Deserialize;
use colored::*;
use std::env;
use std::fs;
use std::path;
use dotenv::dotenv;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Deserialize, Debug)]
struct Query {
    jobs: Vec<Job>,
}
#[derive(Deserialize, Debug)]
struct Job {
    name: String,
    url: String,
    builds: Vec<Build>,
}
#[derive(Deserialize, Debug)]
struct Build {
    number: i32,
    url: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    url: String,
}

lazy_static!{
    static ref URL_REGEX: Regex = Regex::new(r"(\w+://([^/]+))/.+").unwrap();
}
use std::path::PathBuf;
use structopt::StructOpt;


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
/// Allows you browse jenkins from the command line!
#[derive(StructOpt, Debug)]
#[structopt(name = "jenky")]
struct Opt {
    /// A regex pattern to filter job URLs
    #[structopt()]
    pattern: Option<String>,
}

fn main() {
    dotenv().ok();
    let jenkins_url = env::var("JENKINS_URL").expect("JENKINS_URL envionment variable set");
    let jenkins_url = jenkins_url.as_str();
    
    let opt = Opt::from_args();
    let job_url_regex: Option<Regex> = opt.pattern.map(|p| Regex::new(p.as_str())).unwrap().ok();

    let jenkins = JenkinsBuilder::new(jenkins_url.clone())
        .build()
        .unwrap();

    let mut view = jenkins.get_view("All").unwrap();

    let pipelines: Vec<String> = view.jobs.drain(..).map(|j| j.name).collect();

    use jenkins_api::client::TreeBuilder;

    for pipeline in pipelines {
        let path = jenkins_api::client::Path::Job {
            name: pipeline.as_str(),
            configuration: None,
        };
        let tree = TreeBuilder::object("jobs")
            .with_subfield("name")
            .with_subfield("url")
            .with_subfield(
                TreeBuilder::object("builds")
                    .with_subfield("url")
                    .with_subfield("number"),
            )
            .build();
        let query: Result<Query,_> = jenkins.get_object_as(path, tree);
        match query {
            Ok(query) =>{
                // let job_url_regex_ref = job_url_regex.as_ref();
                for job in query.jobs {
                    let nb_builds = job.builds.len();
                    if nb_builds > 0 {
                        let job_url = replace_url(job.url.as_str(), jenkins_url);

                            if let Some(regex) = job_url_regex.as_ref() {
                                if let Some(_) = regex.captures(job.url.as_str()){
                                    println!("{}", job_url.bold().underline());
                                    for build in job.builds {
                                        println!("{:4} | {}", build.number, replace_url(build.url.as_str(), jenkins_url));
                                    }
                                    print!("\n");
                                }
                            }
                        }
                    }
                    // if nb_builds == 0 {
                    //     println!("{}", "no builds".dimmed());
                    // }
            },
            Err(e) => {
                // Ignore the errors
                // eprintln!("{}",e)
            }
        }
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


fn replace_url(url: &str, replacement: &str) -> String {
    let mut url_mut = String::new();
    url_mut.push_str(url);
    if let Some(captures) = URL_REGEX.captures(url_mut.as_str()){
        if let Some(m) = captures.get(1) {
            return url_mut.replace(m.as_str(), replacement);
        }
    }
    url_mut
}