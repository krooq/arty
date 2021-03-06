# Arty
```
Command line based browser and artifact downloader for jenkins!

SUBCOMMANDS:
    get     Download artifacts from jenkins
    help    Prints this message or the help of the given subcommand(s)
```
## get
```
Download artifacts from jenkins

USAGE:
    arty.exe get [OPTIONS] [JOB]

OPTIONS:
    -a, --artifacts <artifacts>
            filter results by artifact name

    -b, --build <build>
            filter results by build number

    -d, --download-dir <download-dir>
            download directory [env: ARTY_DOWNLOADS_DIR=]  [default: .]

    -j, --job <job>
            filter results by job name (will override positional argument)

    -p, --pipeline <pipeline>
            filter results by pipeline name

        --url <url>
            jenkins url [env: ARTY_JENKINS_URL=http://localhost:8080]  [default: http://localhost:8080]

ARGS:
    <JOB>
            filter results by job name
```

NOTE: Querying every artifact of every build would take forever! To work around this, artifacts are only queried once
the search has been filtered to a single build.

# Installing
- Install Rust: https://rustup.rs/
- Run `$ cargo install arty`
- Make sure your cargo bin directory is on the PATH

# Building from source
- Install Rust: https://rustup.rs/
- Run either of the following in the project root:
    - `$ cargo run` to quickly run in debug mode
    - `$ cargo build --release` to assemble an executable in `target/release`

# Limitations
- No support for user credentials (yet)
- No tests
- Might fail with an ugly error message
- Developed for and only "tested" with multi-pipeline branches


# License
Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
   
at your option.

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
