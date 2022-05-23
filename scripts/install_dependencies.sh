#!/bin/bash

sudo amazon-linux-extras install docker
sudo service docker start
sudo usermod -a -G docker ec2-user

docker login -u AWS -p $(aws ecr get-login-password --region $(cat /tmp/ECR_REGION)) $(cat /tmp/ECR_REGISTRY)
