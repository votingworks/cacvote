# CACVote

## Setup

1. Set up a Debian 11 machine (Debian 12 or Ubuntu may work).
2. Clone this repo, `cd` to it.
3. Run the setup script: `script/cacvote-setup`.
4. Install monorepo dependencies:
   - `pnpm install`
   - `cargo build`

## Structure

- `apps/cacvote-mark`: the CACVote Mark voter-facing application
- `apps/cacvote-scan`: the CACVote Scan application for ballot scanning
- `apps/cacvote-jx`: the CACVote Jurisdiction application for election
  management
- `services/cacvote-server`: the CACVote server providing sync services for the
  apps

## Development

The easiest way to run the services in development is with `mprocs` at the
repository root.

> Tip: stop and restart the services in `mprocs` with `x` and `r` respectively.

## Release

Each app/service can be built and run individually. Try `make run` in each
app/service directory. You'll need to specify the configuration either via
environment variables (e.g. `DATABASE_URL`) or command-line flags.

## Logging

Logging for everything but `cacvote-mark` is configured with the `LOG_LEVEL`
environment variable or `--log-level` CLI option. For example:

```sh
LOG_LEVEL=debug mprocs
```

## License

GPLv3
