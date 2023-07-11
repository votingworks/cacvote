DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "${DIR}/.."

sudo service postgresql start

USER=${1:-postgres}
TEST_USER=${2:-rave}
TEST_DB=${3:-rave_test}
TEST_PASSWORD=${4:-test}

cat <<EOS | sudo -u "${USER}" psql -v ON_ERROR_STOP=1
DO \$$
BEGIN
CREATE ROLE ${TEST_USER} LOGIN PASSWORD '${TEST_PASSWORD}';
EXCEPTION WHEN duplicate_object THEN RAISE NOTICE '%, skipping', SQLERRM USING ERRCODE = SQLSTATE;
END
\$$;

DROP DATABASE IF EXISTS ${TEST_DB};
CREATE DATABASE ${TEST_DB};
EOS

export DATABASE_URL="postgres://${TEST_USER}:${TEST_PASSWORD}@localhost/${TEST_DB}"
echo "localhost:*:*:${TEST_USER}:${TEST_PASSWORD}" > ~/.pgpass
chmod 600 ~/.pgpass

cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --source db/migrations
