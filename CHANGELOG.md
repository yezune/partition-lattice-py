# Changelog

## 0.1.1

Prebuilt wheels for Linux and Windows. The library itself is unchanged.

0.1.0 shipped a macOS arm64 wheel and an sdist, so everyone else had to build from
source with a Rust toolchain installed. This release adds:

- `manylinux_2_17` wheels for x86_64 and aarch64
- a `win_amd64` wheel
- a macOS x86_64 wheel

Cross-compiling the Windows wheel needs PyO3's `generate-import-lib`, so the
dependency's feature list changed; that is the only source difference from 0.1.0.

All wheels are `abi3` and cover CPython 3.8 and newer.

## 0.1.0

First release: Python bindings for the
[`partition-lattice`](https://crates.io/crates/partition-lattice) crate.

- `Partition` with `meet` / `join` (also `&` / `|`), the refinement order, and
  label-blind equality and hashing.
- Measures: `logical_entropy`, the exact `dit_count`, `distance`,
  `mutual_information`, `divergence`, `rand_agreement`, `jaccard`.
- Exact rational measures: `weighted_entropy`, `cross_entropy`.
- Module functions `dit_xor_count`, `destr`, `creat`.
- Type stubs and `py.typed`.

Unlike the Rust `from_blocks`, `Partition.from_blocks` rejects a partial or
overlapping cover instead of silently reassigning elements.
