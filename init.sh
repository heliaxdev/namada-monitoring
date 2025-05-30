#!/bin/bash

# Check if CONFIG_URL is set
if [ -z "$CONFIG_URL" ]; then
  echo "CONFIG_URL is not set."
  exit 1
fi

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

# Download the file
curl -fSL "$CONFIG_URL" -o config.toml || {
  echo "Download failed."
  exit 1
}

/app/monitoring --chain-id $CHAIN_ID --rpc $RPC --config-path config.toml