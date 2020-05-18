pipeline {
    agent any
    stages {
        stage('Rustfmt') {
            steps {
                sh "cargo fmt --all -- --check"
            }
        }
        stage('Clippy') {
            steps {
                sh "cargo clippy --all"
            }
        }
        stage('Build') {
            steps {
                sh 'cargo build'
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
