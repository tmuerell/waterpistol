# Waterpistol - A single binary run environment for gatling tests

## How to manually build

Clone the repository:

```
git clone https://github.com/tmuerell/waterpistol.git
```

Install rust:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install WASM target

```
rustup target add wasm32-unknown-unknown
```

Install build essentials (Ubuntu)

```
sudo apt install build-essential
```

Install trunk

```
cargo install trunk
```

