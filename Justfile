set dotenv-load

sync:
  rsync --exclude=target/ -avz . ${RSYNC_TARGET}

dependencies:
  rustup target add wasm32-unknown-unknown
  cargo install trunk
  cargo install cargo-watch

dev:
  #!/usr/bin/env bash
  set -euo pipefail
  IFS=$'\n\t'

  (trap 'kill 0' SIGINT; \
  bash -c 'cd frontend; trunk serve --proxy-backend=http://[::1]:8081/api/' & \
  bash -c 'cargo watch -- cargo run --bin server -- --port 8081 --gatling-dir=${TEST_GATLING_DIR}')

prod:
  #!/usr/bin/env bash
  set -euo pipefail
  IFS=$'\n\t'

  pushd frontend
  trunk build
  popd

  cargo run --bin server --release -- --port 8080  --gatling-dir=${TEST_GATLING_DIR}

prod-build:
  #!/usr/bin/env bash
  set -euo pipefail
  IFS=$'\n\t'

  pushd frontend
  trunk build
  popd

  cargo build --release 