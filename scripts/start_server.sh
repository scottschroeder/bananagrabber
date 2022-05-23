#!/bin/bash

echo "starting version $(cat /tmp/deploy/version.txt)"
# target is defined by the buildspec.yml and should contain the URI
# of the built image

REGION=$(cat /tmp/deploy/ECR_REGION)
APPLICATION_ID=$(aws ssm get-parameter --region $REGION --name /bananagrabber/application_id --with-decryption | jq -r '.Parameter.Value')
GUILD_ID=$(aws ssm get-parameter --region $REGION --name /bananagrabber/guild_id --with-decryption | jq -r '.Parameter.Value')
DISCORD_TOKEN=$(aws ssm get-parameter --region $REGION --name /bananagrabber/discord_token --with-decryption | jq -r '.Parameter.Value')

docker run \
  -e APPLICATION_ID=$APPLICATION_ID \
  -e DISCORD_TOKEN=$DISCORD_TOKEN \
  -e GUILD_ID=$GUILD_ID \
  --rm -d "$(cat /tmp/deploy/ECR_REGISTRY)/$(cat /tmp/deploy/target)"
