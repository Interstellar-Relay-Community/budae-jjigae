#!/bin/bash

docker buildx create --use --name multi-arch-builder
docker buildx build --platform=linux/arm64,linux/amd64 --push -t perillamint/budae-jjigae:latest .
