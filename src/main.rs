use anyhow::{Context, Result};
use dotenv::dotenv;
use jenkins_api::build::Artifact;
use jenkins_api::client::{TreeBuilder, TreeQueryParam};
use jenkins_api::JenkinsBuilder;
use prettytable::{Cell, Table};
use regex::Regex;
use reqwest;
use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;

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
    jobs: Option<Vec<Job>>,
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
/// Command line based browser and artifact downloader for jenkins!
#[derive(StructOpt, Debug)]
#[structopt(name = "arty")]
enum Subcommand {
    /// Download artifacts from jenkins
    ///
    /// NOTE: Querying every artifact of every build would take forever!
    /// To work around this, artifacts are only queried once the search has been filtered to a single build.
    Get(Opt),
}
#[derive(StructOpt, Debug)]
#[structopt(name = "options")]
struct Opt {
    /// jenkins url
    #[structopt(
        long,
        env = "ARTY_JENKINS_URL",
        default_value = "http://localhost:8080"
    )]
    url: String,

    /// download directory
    #[structopt(short, long, env = "ARTY_DOWNLOADS_DIR", default_value = ".")]
    download_dir: PathBuf,

    /// filter results by job name
    #[structopt(name = "JOB")]
    job_default: Option<String>,

    /// filter results by pipeline name
    #[structopt(short, long)]
    pipeline: Option<String>,

    /// filter results by job name (will override positional argument)
    #[structopt(short, long)]
    job: Option<String>,

    /// filter results by build number
    #[structopt(short, long)]
    build: Option<String>,

    /// filter results by artifact name
    #[structopt(short, long)]
    artifacts: Option<String>,
}

fn unwrap_as_regex(regex: &Option<String>) -> Result<Regex> {
    match regex {
        Some(re) => Regex::new(re.as_str()).context("Invalid regex"),
        None => Regex::new(".*").context("Error in program, contact developer."),
    }
}

fn main() -> Result<()> {
    dotenv().context("Error in .env config file")?;

    match Subcommand::from_args() {
        Subcommand::Get(opt) => {
            let url = &opt.url;
            let download_dir = &opt.download_dir;
            let pipeline_regex: Regex = unwrap_as_regex(&opt.pipeline)?;
            let job_regex: Regex = unwrap_as_regex(&opt.job.or(opt.job_default))?;
            let build_regex: Regex = unwrap_as_regex(&opt.build)?;
            let artifact_regex: Regex = unwrap_as_regex(&opt.artifacts)?;

            let jenkins = JenkinsBuilder::new(url.as_str()).build().unwrap();

            let home = jenkins
                .get_object_as::<_, Home>(jenkins_api::client::Path::Home, metadata_query())
                .expect("Request data from jenkins");

            let mut search_results: Vec<SearchResult> = Vec::new();
            for pipeline in home.pipelines {
                if pipeline_regex.is_match(&pipeline.name) {
                    if let Some(jobs) = pipeline.jobs {
                        for job in jobs {
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
            }
            search_results.sort_by(|a, b| {
                match a
                    .pipeline_name
                    .to_lowercase()
                    .cmp(&b.pipeline_name.to_lowercase())
                {
                    std::cmp::Ordering::Equal => {
                        match a.job_name.to_lowercase().cmp(&b.job_name.to_lowercase()) {
                            std::cmp::Ordering::Equal => b.build_number.cmp(&a.build_number),
                            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                        }
                    }
                    std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                }
            });
            let ref mut artifacts: Vec<Artifact> = Vec::new();
            let mut artifact_path = String::from("");
            if opt.artifacts.is_some() {
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
                    println!("No builds found, try expanding your filters");
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
            for result in &search_results {
                let mut content_row =
                    row![result.pipeline_name, result.job_name, result.build_number];
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

            if opt.artifacts.is_some() {
                if search_results.len() as u32 == 1 {
                    if !artifacts.is_empty() {
                        if yn(&format!(
                            "{} artifacts found, download all ?",
                            artifacts.len(),
                        )) {
                            println!("downloading...");
                            for artifact in artifacts {
                                let mut artifact_url = String::from("");
                                artifact_url.push_str(&url);
                                artifact_url.push_str(&artifact_path);
                                artifact_url.push_str("/artifact/");
                                artifact_url.push_str(&artifact.relative_path);

                                let mut resp = reqwest::blocking::get(&artifact_url)?;
                                let mut file_dir = download_dir.clone();
                                file_dir.push(std::path::Path::new(&artifact.file_name));
                                let mut out = std::fs::File::create(&file_dir)?;
                                std::io::copy(&mut resp, &mut out)?;
                            }
                            println!("done!");
                        }
                    } else {
                        println!("No artifacts found");
                    }
                }
            }
        }
    };
    Ok(())
}

fn yn(yes_no_question: &str) -> bool {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    print!("{}: ", yes_no_question);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s == "y"
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
