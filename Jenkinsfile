pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                sh """
                source /mnt/c/Users/Krooq/.bash_profile
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