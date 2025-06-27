#!/usr/bin/bash

set -a
source .env
set +a

if [[ -z "$1" ]]; then
	echo "Usage: $0 <authorization_code>"
	echo "You can get the code by visiting:"
	echo "https://accounts.spotify.com/authorize?client_id=$SPOTIFY_ID&response_type=code&redirect_uri=$SPOTIFY_REDIRECT&scope=user-read-playback-state"
	exit 1
fi

AUTH_CODE=$1

AUTH_HEADER=$(echo -n "$SPOTIFY_ID:$SPOTIFY_SECRET" | base64 | tr -d '\n')
echo $AUTH_HEADER

curl -X POST https://accounts.spotify.com/api/token \
  -H "Authorization: Basic $AUTH_HEADER" \
  -d grant_type=authorization_code \
  -d code="$AUTH_CODE" \
  -d redirect_uri="$SPOTIFY_REDIRECT"
