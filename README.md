# RAVE

## Setup

1. Set up a Debian 11 machine (Debian 12 or Ubuntu may work).
2. Install postgresql and make your user's PG role a superuser.
3. Install Rust > v1.70.0 and WASM support via `rustup`.
   - Install from here: https://rustup.rs/
   - `rustup target add wasm32-unknown-unknown`
4. Install NodeJS v16.19.1 (other v16.x may work).
5. Install some dev tools:
   - `cargo install --locked dioxus-cli`
   - `cargo install --locked sqlx-cli`
   - `cargo install --locked mprocs`
   - `cargo install --locked cargo-watch`
   - `npm install -g pnpm@8.1.0`
6. Clone this repo, `cd` to it.
7. Run `script/reset-db.sh` to setup the databases.
8. Install monorepo dependencies:
   - `pnpm install`
   - `cargo build`

## Running

The easiest way to run the services is with `mprocs`.

> Tip: stop and restart the services in `mprocs` with `x` and `r` respectively.

### Logging

Logging for everything but `rave-mark` is configured with the `RUST_LOG`
environment variable. For example:

```sh
RUST_LOG=info mprocs
```

See the [`env_logger` docs](https://docs.rs/env_logger/0.10.0/env_logger/) for
more info.

### Development

Run all the services in development mode with `mprocs`.

### Release

Run all the services in production mode with `mprocs -c mprocs-release.yaml`.

## License

GPLv3
