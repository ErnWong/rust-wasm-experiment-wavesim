# A Little Experiment: Using Rust via WebAssembly

What is this? Nothing much really. Just a simple 1D wave simulation using RK4 to solve the ODE. The rust code contains the simulation and some code to plot the points into a region of memory that is captured in an `ImageData` on the JavaScript side, and the JavaScript code puts that image onto the canvas every frame.

## Setup

Some instructions valid at the time this was written:

First: Get Rust

```sh
curl https://sh.rustup.rs -sSf | sh
```

Then: Get Rust Nightly

```sh
rustup toolchain install nightly
```

Then: Get the wasm target

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Then: Get `wasm-gc`

```sh
cargo install --git https://github.com/alexcrichton/wasm-gc
```

Done. It should now be possible to build this experiment via

```sh
make
```

Then, you'll need to run a small server to view `index.html` without getting CORS problems (unless there's a better way).
