use colored::*;
use dotenv::dotenv;
use jenkins_api::client::{TreeBuilder, TreeQueryParam};
use jenkins_api::JenkinsBuilder;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::io;

#[derive(Deserialize, Debug)]
struct Pipeline {
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
    artifacts: Vec<Artifact>,
    building: Option<bool>,
    number: i32,
    url: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Artifact {
    // displayPath: String,
    file_name: String,
    relative_path: String,
}
// impl Into<&str> for Artifact {
//     fn into(self) -> &str {
//         displ
//     }
// }
#[derive(Deserialize, Debug)]
struct Config {
    url: String,
}

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"(\w+://([^/]+))/.+").unwrap();
}

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

/// Command line browser and artifact downloader for jenkins!
#[derive(StructOpt, Debug)]
#[structopt(name = "jenky")]
struct Opt {
    /// A regex pattern to filter pipelines
    #[structopt(short, long)]
    pipeline: Option<String>,

    /// A regex pattern to filter jobs
    #[structopt(short, long)]
    job: Option<String>,

    /// A regex pattern to filter builds
    #[structopt(short, long)]
    build: Option<String>,

    /// A regex pattern to filter artifacts
    #[structopt(short, long)]
    artifact: Option<String>,
}

fn main() -> io::Result<()> {
    dotenv().ok();
    let url = env::var("JENKINS_URL").expect("JENKINS_URL envionment variable set");
    let opt = Opt::from_args();
    let job_regex: Option<Regex> = opt.job.map(|p| Regex::new(p.as_str()).unwrap());

    let jenkins = JenkinsBuilder::new(url.as_str()).build().unwrap();

    let mut view = jenkins.get_view("All").unwrap();

    let pipeline_names: Vec<String> = view.jobs.drain(..).map(|j| j.name).collect();

    for pipeline_name in pipeline_names {
        let path = jenkins_api::client::Path::Job {
            name: pipeline_name.as_str(),
            configuration: None,
        };
        let tree = query();
        // dbg!(&tree);
        if let Ok(pipeline) = jenkins.get_object_as::<_, Pipeline>(path, tree) {
            println!("{}", pipeline_name);
            if pipeline.jobs.is_empty() {
                println!("{:8}", "none".dimmed());
            } else {
                for (j, job) in pipeline.jobs.iter().enumerate() {
                    if job_regex
                        .as_ref()
                        .map_or(true, |r| r.captures(job.name.as_str()).is_some())
                    {
                        println!("{:4} | {}", j, job.name.replace("%2F", "/"));
                    }
                }
            }
            println!();
        };
    }

    println!("http://your.jenkins.server/job/your.job/lastStableBuild/artifact/relativePath");

    // let mut input = String::new();
    // println!("You typed: {}", input.trim());
    // io::stdin().read_line(&mut input)?;
    // println!("You typed: {}", input.trim());
    Ok(())

    // let job_url = replace_url(job.url.as_str(), jenkins_url);
    // println!("{:4} | {}", "", job_url.replace("%2F", "/"));
    // println!("{:4} | {}", "", job.builds.iter().map(|b| b.number.to_string()).collect::<Vec<String>>().join(", "));

    // for build in job.builds.iter() {
    //     println!("{:4} | {}", build.number, replace_url(build.url.as_str(), jenkins_url));
    // }
}

fn query() -> TreeQueryParam {
    TreeBuilder::object("jobs")
        .with_subfield("name")
        .with_subfield("url")
        .with_subfield(
            TreeBuilder::object("builds")
                .with_subfield(
                    TreeBuilder::object("artifacts")
                        .with_subfield("fileName")
                        .with_subfield("relativePath"),
                )
                .with_subfield("building")
                .with_subfield("url")
                .with_subfield("number"),
        )
        .build()
}

// fn replace_url(url: &str, replacement: &str) -> String {
//     let mut url_mut = String::new();
//     url_mut.push_str(url);
//     if let Some(captures) = URL_REGEX.captures(url_mut.as_str()) {
//         if let Some(m) = captures.get(1) {
//             return url_mut.replace(m.as_str(), replacement);
//         }
//     }
//     url_mut
// }
