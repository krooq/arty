use anyhow::{Context, Result};
use dotenv::dotenv;
use jenkins_api::build::Artifact;
use jenkins_api::client::{TreeBuilder, TreeQueryParam};
use jenkins_api::JenkinsBuilder;
use prettytable::{Cell, Table};
use regex::Regex;
use reqwest;
use serde::Deserialize;
use std::env;

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
    download_dir: Option<String>,
}

use structopt::StructOpt;

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
    dotenv().context("Error in .env config file")?;
    let url = env::var("JENKY_URL").context("JENKY_URL envionment variable not set")?;
    let download_dir = env::var("JENKY_DOWNLOAD_DIR").ok().map_or_else(
        || std::env::current_dir().unwrap(),
        |dir| std::path::PathBuf::from(dir),
    );

    let opt = Opt::from_args();

    let pipeline_regex: Regex = unwrap_as_regex(&opt.pipeline)?;
    let job_regex: Regex = unwrap_as_regex(&opt.job)?;
    let build_regex: Regex = unwrap_as_regex(&opt.build)?;
    let artifact_regex: Regex = unwrap_as_regex(&opt.artifact)?;

    let jenkins = JenkinsBuilder::new(url.as_str()).build().unwrap();

    let home = jenkins
        .get_object_as::<_, Home>(jenkins_api::client::Path::Home, metadata_query())
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

    let ref mut artifacts: Vec<Artifact> = Vec::new();
    let mut artifact_path = String::from("");
    if opt.artifact.is_some() {
        if !search_results.is_empty() {
            if search_results.len() as u32 == 1 {
                let result = search_results.first().unwrap();
                artifact_path.push_str("/job/");
                artifact_path.push_str(&result.pipeline_name.clone());
                artifact_path.push_str("/job/");
                artifact_path.push_str(&result.job_name);
                artifact_path.push_str("/");
                artifact_path.push_str(&result.build_number.to_string());
                let build: jenkins_api::build::CommonBuild = jenkins
                    .get_object_as(
                        jenkins_api::client::Path::Raw {
                            path: &artifact_path,
                        },
                        None,
                    )
                    .expect("Artifact data from jenkins");
                for artifact in build.artifacts {
                    if artifact_regex.is_match(&artifact.file_name) {
                        artifacts.push(artifact);
                    }
                }
            } else {
                println!("Mutiple builds found, artifacts can only be retrieved for exactly one build at a time, try refining your filters");
            }
        } else {
            println!("No builds found, artifacts can only be retrieved for exactly one build at a time, try expanding your filters");
        }
    }

    let mut table = Table::new();
    let mut header_row = row![c=>"pipeline", "job", "build"];
    if !artifacts.is_empty() {
        let mut artifacts_header = Cell::new("artifacts");
        artifacts_header.align(prettytable::format::Alignment::CENTER);
        header_row.add_cell(artifacts_header);
    }
    table.add_row(header_row);
    for result in search_results {
        let mut content_row = row![result.pipeline_name, result.job_name, result.build_number];
        if !artifacts.is_empty() {
            content_row.add_cell(Cell::new(
                &artifacts
                    .into_iter()
                    .map(|a| a.relative_path.clone())
                    .collect::<Vec<String>>()
                    .join("\n"),
            ));
        }
        table.add_row(content_row);
    }
    table.printstd();

    if opt.artifact.is_some() {
        if !artifacts.is_empty() {
            if artifacts.len() as u32 == 1 {
                let artifact = artifacts.first().unwrap();
                let mut artifact_url = String::from("");
                artifact_url.push_str(&url);
                artifact_url.push_str(&artifact_path);
                artifact_url.push_str("/artifact/");
                artifact_url.push_str(&artifact.relative_path);

                let mut resp = reqwest::blocking::get(&artifact_url)?;
                let mut file_dir = download_dir;
                file_dir.push(std::path::Path::new(&artifact.file_name));
                let mut out = std::fs::File::create(&file_dir)?;
                std::io::copy(&mut resp, &mut out)?;
            }
        } else {
            println!("No artifacts found");
        }
    }
    Ok(())
}

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
                        .with_field("building")
                        .with_field("url")
                        .with_field("number"),
                ),
        )
        .build()
}
