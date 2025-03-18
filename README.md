# CACvote

## Production TODOs

This is a prototype and there are a variety of things that need to be done to
make it production-ready. Here are some of the most important:

1. **Validation of production CACs**: Real CACs use a different certificate
   authority than the one we're using for development. We need to validate
   against the DoD's PKI infrastructure.
2. **Security audit**: We have done work to secure the system, in particular
   with the use of Java cards for authentication and validation of client
   certificates stored in the TPM for requests to `cacvote-server`. However, we
   have not performed a full security audit.
3. **Scale testing**: We have not tested the system at scale. We need to ensure
   that it can handle the load of a real election across multiple jurisdictions.

## Setup

1. Set up a Debian 11 machine (Debian 12 or Ubuntu may work).
2. Allow your user to run as root:

```
$ su
root# echo "your-username ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers.d/your-username
```

Be sure to replace `your-username` with your actual username. Then open a new
Terminal window and run `sudo ls` to ensure that you can run commands as root
without a password. Note that this is not recommended for production systems.

3. Open a Terminal and install git: `sudo apt-get install -y git`.
4. Clone this repo, `cd` to it:
   `git clone https://github.com/votingworks/cacvote.git && cd cacvote`.
5. Run the setup script: `script/cacvote-setup`. This will take a while.

## Structure

- `apps/cacvote-mark`: the CACvote Mark voter-facing application
- `apps/cacvote-jx`: the CACvote Jurisdiction application for election
  management
- `apps/cacvote-server`: the CACvote server providing sync services for the apps

## Configuration

Configuration is done via environment variables. The following are required, but
should automatically be provided for you if you use `mprocs`:

- `EG_CLASSPATH`: the path to the `egk-ec-mixnet` uberJar. It should be in a
  `egk-ec-mixnet` directory in this repo's parent directory, e.g.
  `$HOME/egk-ec-mixnet/build/egk-ec-mixnet-2.1-SNAPSHOT-uber.jar`

### Scanning Mail Label QR Codes

When running the CACvote Server locally, you'll need a way to expose the server
to the internet so that your mobile device can connect to it. One way to do this
is with `ngrok` (requires signup). Once you've got `ngrok` installed you can run
the `expose-server` process when using `mprocs` with the
`mprocs-cacvote-server.yaml` configuration. The URL to connect to from your
phone will be printed to the console and will look something like this:

```
Forwarding https://dd54-167-131-35-180.ngrok-free.app -> http://localhost:3000
```

## Development

The easiest way to run the services in development is with `mprocs` at the
repository root:

> Tip: stop and restart the services in `mprocs` with `x` and `r` respectively.

- CACvote Mark (Voter Terminal): Run with `mprocs -c mprocs-cacvote-mark.yaml`
  - In order to authenticate, you'll need either a real or test Common Access
    Card (CAC), or a mock card. To create a mock card, insert a Java card and
    run the following command:
    ```sh
    $ CERT_COMMON_NAME=LAST.FIRST.MIDDLE.0123456789 ./libs/auth/scripts/cac/configure-dev-simulated-cac-card
    ```
    Be sure that you've plugged in the card reader and that you've forwarded the
    reader to the VM if applicable. This mock card will have a PIN of `77777777`
    (8 sevens).
- CACvote Jurisdiction (Jurisdiction Terminal): Run with
  `mprocs -c mprocs-cacvote-jx-terminal.yaml`
  - In order to authenticate, you'll need a VotingWorks system administrator
    card with the right jurisdiction field. To create a card for use in
    development, run the following command:
    ```sh
    $ ./libs/auth/scripts/program-dev-system-administrator-java-card
    ```
    Be sure that you've plugged in the card reader and that you've forwarded the
    reader to the VM if applicable.
- CACvote Server: Run with `mprocs -c mprocs-cacvote-server.yaml`
- Usability Test (CACvote Mark only): Run with
  `mprocs -c mprocs-usability-test.yaml`

Note that each of the `mprocs` configurations have processes that do not start
immediately, but can be started manually to perform certain functions such as
resetting the database.

### Printing

Ensure you've plugged in the printers and that they're powered on. If you don't
you'll get errors when attempting to print.

To print with PDF, run this:

```sh
$ echo REACT_APP_VX_USE_MOCK_PRINTER=TRUE >> apps/cacvote-mark/backend/.env.local
```

When printing, the ballot PDF prints will be saved to
`libs/fujitsu-thermal-printer/dev-workspace/prints` and the mail label PDF
prints will be saved to `apps/cacvote-mark/backend/dev-workspace/prints`.

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
