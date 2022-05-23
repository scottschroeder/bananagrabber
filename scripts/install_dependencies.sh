#!/bin/bash

sudo amazon-linux-extras install docker
sudo service docker start
sudo usermod -a -G docker ec2-user

echo "we are in the install script"
pwd
env
# aws ecr get-login-password --region "$(cat ECR_REGION)" | docker login --username AWS --password-stdin "$(cat ECR_REGISTRY)"
docker login -u AWS -p $(aws ecr get-login-password --region $(cat /tmp/ECR_REGION)) $(cat /tmp/ECR_REGISTRY)
