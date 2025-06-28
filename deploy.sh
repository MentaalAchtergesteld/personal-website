#!/usr/bin/bash

set -e


PI_USER="mentaal"
PI_HOST="raspberrypi"
REMOTE_DIR="/home/mentaal/website"
BINARY_NAME="personal-website"
BINARY_LOCATION="armv7-unknown-linux-musleabihf"

echo "Stopping service..."
ssh $PI_USER@$PI_HOST "sudo systemctl stop personal-website"

echo "Copying compiled binary..."
scp target/$BINARY_LOCATION/release/$BINARY_NAME \
    $PI_USER@$PI_HOST:$REMOTE_DIR/$BINARY_NAME
scp .env $PI_USER@$PI_HOST:$REMOTE_DIR

echo "Synchronizing static files..."
rsync -av --delete static/ $PI_USER@$PI_HOST:$REMOTE_DIR/static

echo "Restarting service..."
ssh $PI_USER@$PI_HOST "sudo systemctl restart personal-website"

echo "Deployed!"
