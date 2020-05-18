pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                sh 'cargo build'
            }
        }
        stage('Clippy') {
            steps {
                sh "cargo clippy --all"
            }
        }
        stage('Rustfmt') {
            steps {
                sh "cargo fmt --all -- --check"
            }
        }
        stage('Test') {
            steps {
                sh "cargo test"
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'target/**/*'
        }
    }
}
