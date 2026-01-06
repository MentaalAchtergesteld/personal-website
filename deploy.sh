#!/usr/bin/bash

PI_USER="pi"
PI_HOST="raspberrypi.local"
DIR="~/website"

echo "[1/4] Building container..."
docker buildx build --platform linux/arm64 -t pi-site:latest --load .

echo "[2/4] Uploading to Raspberry PI"
docker save pi-site:latest | ssh -C $PI_USER@$PI_HOST "docker load"

echo "[3/4] Syncinv .env."
scp .env $PI_USER@$PI_HOST:$DIR/.env

echo "[4/4] Restarting container..."
ssh $PI_USER@$PI_HOST "cd $DIR && docker compose up -d --force-recreate app"

echo "[-/-] Finished."
