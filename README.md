# Waterpistol - A single binary run environment for gatling tests

Waterpistol is a remote runner for Gatling tests (<https://gatling.io/>)

## How to use it

* Download the correct binary to the testrunner host
* Run the binary application: `./waterpistol -a 0.0.0.0 -p 8080 --data-dir data`
* This opens up a HTTP server on the given IP address and port
* Open this url in a browser
* Upload a gatling testsuite
* Execute a testrun
* View the results

## How to manually build

Clone the repository:

```bash
git clone https://github.com/tmuerell/waterpistol.git
```

Install rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install WASM target

```bash
rustup target add wasm32-unknown-unknown
```

Install build essentials (Ubuntu)

```bash
sudo apt install build-essential
```

Install trunk

```bash
cargo install trunk
```
