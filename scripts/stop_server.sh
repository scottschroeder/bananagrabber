#!/bin/bash

if ! command -v docker &> /dev/null 
then
    echo "docker is not installed"
    exit 0 # nothing to stop
fi

docker kill $(docker ps -q)
docker image prune -af
