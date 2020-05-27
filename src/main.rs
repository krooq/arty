use anyhow::{Context, Result};
use colored::*;
use dotenv::dotenv;
use jenkins_api::build::{Artifact, BuildNumber};
use jenkins_api::client::{TreeBuilder, TreeQueryParam};
use jenkins_api::job::JobName;
use jenkins_api::JenkinsBuilder;
use lazy_static::lazy_static;
use prettytable::{Cell, Row, Table};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::io;

#[macro_use]
extern crate prettytable;
#[derive(Deserialize, Debug)]
struct Home {
    #[serde(rename(deserialize = "jobs"))]
    pipelines: Vec<Pipeline>,
}
#[derive(Deserialize, Debug)]
struct Pipeline {
    name: String,
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
    building: bool,
    number: u32,
    url: String,
}

struct SearchResult {
    pipeline_name: String,
    job_name: String,
    build_number: u32,
}
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

    /// A regex pattern to filter build artifacts
    /// For performance, artifacts can only be retrieved for one build at a time
    #[structopt(short, long)]
    artifact: Option<String>,
}

fn unwrap_as_regex(regex: &Option<String>) -> Result<Regex> {
    match regex {
        Some(re) => Regex::new(re.as_str()).context("Invalid regex"),
        None => Regex::new(".*").context("Error in program, contact developer."),
    }
}

fn main() -> Result<()> {
    dotenv().ok();
    let url = env::var("JENKINS_URL").expect("Get JENKINS_URL envionment variable");
    let opt = Opt::from_args();

    let pipeline_regex: Regex = unwrap_as_regex(&opt.pipeline)?;
    let job_regex: Regex = unwrap_as_regex(&opt.job)?;
    let build_regex: Regex = unwrap_as_regex(&opt.build)?;
    let artifact_regex: Regex = unwrap_as_regex(&opt.artifact)?;

    let jenkins = JenkinsBuilder::new(url.as_str()).build().unwrap();

    // let mut filtered_results = Vec::new();

    let tree = metadata_query();
    let home = jenkins
        .get_object_as::<_, Home>(jenkins_api::client::Path::Home, tree)
        .expect("Request data from jenkins");

    let mut search_results: Vec<SearchResult> = Vec::new();
    for pipeline in home.pipelines {
        if pipeline_regex.is_match(&pipeline.name) {
            for job in pipeline.jobs {
                if job_regex.is_match(&job.name) {
                    for build in job.builds {
                        if build_regex.is_match(&build.number.to_string()) {
                            search_results.push(SearchResult {
                                pipeline_name: pipeline.name.clone(),
                                job_name: job.name.clone(),
                                build_number: build.number,
                            });
                        }
                    }
                }
            }
        }
    }

    let mut artifacts: Vec<Artifact> = Vec::new();
    if opt.artifact.is_some() {
        if search_results.len() as u32 == 1 {
            let result = search_results.first().unwrap();
            let mut job_name = result.pipeline_name.clone();
            job_name.push_str("/job/");
            job_name.push_str(&result.job_name);
            let build = jenkins
                .get_build(
                    JobName::from(&job_name),
                    BuildNumber::Number(result.build_number),
                )
                .expect("Artifact data from jenkins");
            for artifact in build.artifacts {
                if artifact_regex.is_match(&artifact.file_name) {
                    artifacts.push(artifact);
                }
            }
        } else {
            println!("Mutiple builds found, artifacts can only be retrieved for one build at a time, refine filters to query build artifacts");
        }
    }

    let mut table = Table::new();
    table.add_row(row![c=>"pipeline", "job", "build"]);
    for result in search_results {
        table.add_row(row![
            result.pipeline_name,
            result.job_name,
            result.build_number
        ]);
    }
    table.printstd();

    if !artifacts.is_empty() {
        println!("Artifacts: ");
        for artifact in artifacts {
            println!("{}", artifact.file_name);
        }
    }
    // println!("http://your.jenkins.server/job/your.job/lastStableBuild/artifact/relativePath");

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

// fn column_format<I: IntoIterator<Item = String>>(values: I) -> String {
//     let pipeline_column_width = values
//         .into_iter()
//         .map(|v| v.chars().count())
//         .max()
//         .unwrap_or(0);
//     let fmt = String::from("{%");
//     fmt.push_str(&pipeline_column_width.to_string());
//     fmt.push_str("s}");
//     fmt
// }

/// Builds a request for obtaining shallow metadata on the jenkins server.
fn metadata_query() -> TreeQueryParam {
    TreeBuilder::object("jobs")
        .with_field("name")
        .with_field(
            TreeBuilder::object("jobs")
                .with_field("name")
                .with_field("url")
                .with_field(
                    TreeBuilder::object("builds")
                        // .with_field(
                        //     TreeBuilder::object("artifacts")
                        //         .with_field("fileName")
                        //         .with_field("relativePath"),
                        // )
                        .with_field("building")
                        .with_field("url")
                        .with_field("number"),
                ),
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
