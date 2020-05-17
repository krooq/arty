pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                sh """#!/bin/bash -l
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