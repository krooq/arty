pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                withEnv(['PATH+CARGO_HOME=/home/krooq/main/tools/rust/cargo/bin']) {
                    sh 'whoami'
                    sh 'cargo build'
                }
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'target/*/*.exe'
        }
    }
}
