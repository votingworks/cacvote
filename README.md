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
- `apps/cacvote-jx`: the CACVote Jurisdiction application for election
  management
- `apps/cacvote-server`: the CACVote server providing sync services for the apps

## Configuration

Configuration is done via environment variables. The following are required:

- `EG_CLASSPATH`: the path to the `egk-ec-mixnet` uberJar. It should be in a
  `egk-ec-mixnet` directory in this repo's parent directory, e.g.
  `$HOME/egk-ec-mixnet/build/egk-ec-mixnet-2.1-SNAPSHOT-uber.jar`

### Database Preparation

In order to scan mail label QR codes, the Voter Terminal machines will need to
be enrolled. In the `cacvote-server` directory, run the following command:

```sh
$ cargo run --bin enroll-voter-terminal-machine MACHINE_ID PATH_TO_CERT
```

For example, when using a development certificate:

```sh
$ cargo run --bin enroll-voter-terminal-machine 000 ../../../libs/auth/certs/dev/vx-mark-cert.pem
✅ Machine enrolled! ID=08209a41-b0e1-405e-ae65-2becff7eb848
```

Or when using a production certificate:

```sh
$ cargo run --bin enroll-voter-terminal-machine some-vx-voter-terminal-432 path/to/tpm-public-cert.pem
✅ Machine enrolled! ID=08209a41-b0e1-405e-ae65-2becff7eb848
```

## Development

The easiest way to run the services in development is with `mprocs` at the
repository root. Be sure to include the `EG_CLASSPATH` environment variable!

> Tip: stop and restart the services in `mprocs` with `x` and `r` respectively.

You can use `mprocs` with the `mprocs-usability-test.yaml` configuration file to
run just the CACVote Mark app in a way that's suitable for usability testing:

```sh
mprocs -c mprocs-usability-test.yaml
```

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
