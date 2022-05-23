#!/bin/bash

echo "starting version $(cat version.txt)"
# target is defined by the buildspec.yml and should contain the URI
# of the built image
docker run --rm "$(cat ECR_REGISTRY):$(cat target)"
