#!/bin/bash

# Check if CHAIN_ID is set
if [ -z "$CHAIN_ID" ]; then
  echo "CHAIN_ID is not set."
  exit 1
fi

# Check if RPC is set
if [ -z "$RPC" ]; then
  echo "RPC is not set."
  exit 1
fi

# Check if CONFIG_PATH is set
if [ -n "$CONFIG_PATH" ]; then
  if [ ! -f "$CONFIG_PATH" ]; then
    echo "CONFIG_PATH file does not exist."
    exit 1
  fi
  CONFIG_FILE="$CONFIG_PATH"
elif [ -n "$CONFIG_URL" ]; then
  curl -fSL "$CONFIG_URL" -o config.toml || {
    echo "Download failed."
    exit 1
  }
  CONFIG_FILE="config.toml"
else
  echo "Neither CONFIG_PATH nor CONFIG_URL is set."
  exit 1
fi

/app/monitoring --chain-id $CHAIN_ID --rpc $RPC --config-path $CONFIG_FILE