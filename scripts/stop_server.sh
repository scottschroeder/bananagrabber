#!/bin/bash

docker kill $(docker ps -q)
docker image prune -af
