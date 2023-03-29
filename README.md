APP_ENV: specifies which environment the app runs in. If not specified, `local` is assumed.
TEST_LOG: if specified, logs emitted during tests will be printed to stdout. If not specified, logs are dropped.

POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_DB, POSTGRES_PORT: variables configuring the DB container. Default values can be found in `./scripts/init-db.sh`.

SKIP_DOCKER: If is specified, running `./scripts/init-db.sh` doesn't setup the DB docker container.
