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