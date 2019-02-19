#!/bin/bash
docker run -p 9000:9000 \
  --name minio1 --rm \
  --volume "$(pwd)"/objects:/export \
  --volume "$(pwd)"/.minio:/root/.minio \
  --env MINIO_ACCESS_KEY=abc1 \
  --env MINIO_SECRET_KEY=12345678 \
  minio/minio server /export
