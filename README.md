# RAVE

## Setup

1. Set up a Debian 11 machine (Debian 12 or Ubuntu may work).
2. Install postgresql and make your user's PG role a superuser.
3. Install Rust > v1.70.0 via `rustup`.
4. Install NodeJS v16.19.1 (other v16.x may work).
5. Install some dev tools:
   - `cargo install --locked mprocs`
   - `cargo install --locked cargo-watch`
   - `npm install -g pnpm@8.1.0`
6. Clone this repo, `cd` to it.
7. Install monorepo dependencies:
   - `pnpm install`
   - `cargo build`
8. Run `script/reset-db.sh` to setup the databases.
9. Run all the services in development mode with `mprocs`.
10. Run a specific service, e.g. `rave-scan`, by running
    `RAVE_URL=https://rave.example.com/ VX_MACHINE_ID=rave-scan-000 pnpm --dir apps/rave-scan/frontend start`,
    replacing the URL and machine ID as appropriate.

## License

GPLv3
