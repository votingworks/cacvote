#!/usr/bin/env bash

set -euo pipefail

(
  echo "♻️ rave-server…"
  cd services/rave-server
  sqlx database reset --source db/migrations
)

(
  echo "♻️ rave-scan…"
  cd apps/rave-scan/backend
  sqlx database reset --source db/migrations
)

(
  echo "♻️ rave-jx…"
  cd apps/rave-jx-terminal/backend
  sqlx database reset --source db/migrations
)

(
  echo "♻️ rave-mark…"
  cd apps/rave-mark/backend
  rm -rf dev-workspace
)

echo "✅ dbs reset"
