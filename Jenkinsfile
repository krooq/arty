pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                sh """
                echo "$HOME"
                bash
                echo "$HOME"
                source /mnt/c/Users/Krooq/.bash_profile
                echo "$HOME"
                cargo build
                """
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'target/*/*.exe'
        }
    }
}