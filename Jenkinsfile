withEnv(['PATH+CARGO_HOME=/home/krooq/main/tools/rust/cargo/bin']) {
    pipeline {
        agent any
        stages {
            stage('Build') {
                steps {
                    sh 'cargo build'
                }
            }
        }
        post {
            always {
                archiveArtifacts artifacts: 'target/*/*.exe'
            }
        }
    }
}