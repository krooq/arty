## arty
Command line based browser and artifact downloader for jenkins!
```
SUBCOMMANDS:
    get     Download artifacts from jenkins
    help    Prints this message or the help of the given subcommand(s)
```

### get
Download artifacts from jenkins

```
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