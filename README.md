# Ariel Wasm runtimes size comparisons

This repository contains a simple out-of-tree application using Ariel OS that loads and runs a simple wasm capsule using different runtimes, allowing to compare the sizes of the resulting code.

## How to run

Three runtimes are supported and can be selected through laze modules.
```shell
# change -s wasmtime to wasmi or wasm-interpeter to select the other runtimes
laze build -b nrf52840dk -s wasmtime run
```
To examine the ELF, replace `run` by `size -A` to get the breakdown of the code size per sections and `bloat --crates` to get the breakdown per crate.
To use `bloat` you need to first do `rustup override set nightly`.