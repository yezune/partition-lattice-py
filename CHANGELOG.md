# Changelog

## 0.2.0 — breaking

**Removed `meet`, `join`, `&`, `|`, and the order comparisons `<= < >= >`.**
Use `refine`, `coarsen` and `refines` instead. Equality is unchanged.

### Why removal rather than deprecation

The two conventions in the literature disagree about which lattice operation is the
common refinement. This package's `meet` was the refinement; in Ellerman's papers the
refinement is the *join*. A reader porting a formula from that side would have written
`a.join(b)`, got a valid result, and never learned it was the wrong operation.

Deprecation does not help: a deprecated `meet` still returns an answer. The only
change that turns a silent wrong result into a visible failure is taking the name
away. `a.meet(b)` now raises `AttributeError`, `a & b` and `a <= b` raise `TypeError`.

`&` and `|` went with them for the same reason — they carry the identical ambiguity in
symbols instead of words — and so did the order comparisons, since which direction
`<=` runs is precisely what the conventions disagree on. `refines()` states the
relation in words that read the same either way.

Equality stayed: whether two partitions are the same partition is not a question the
convention has an opinion about.

### The measurement behind this

Renaming the operations is safe, and that is not an assumption. The decision procedure
this project uses for lattice order was checked exhaustively:

    s <= t  <=>  dual(t) <= dual(s)      6,055 terms, 36,663,025 pairs, 0 mismatches

So the two conventions are exact mirrors: every true statement maps to a true
statement. That is also why the *convention* was not switched — switching buys only
convenience when reading papers, and costs every existing caller a silent reversal.
The naming, not the convention, was what needed fixing.


## 0.1.2

Adds `Partition.refine()` and `Partition.coarsen()`, matching the Rust crate: the same
two operations named for what they do rather than for their position in the lattice.

`meet` and `join` swap meaning between the two conventions in the literature, so a
formula ported without checking which one is in force silently computes the wrong
thing. `refine` and `coarsen` read identically under either.

Pure delegation to `meet` / `join`; nothing else changes.

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
