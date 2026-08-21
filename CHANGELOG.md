# Changelog

## 0.3.2

Three functions exposed, all additive. Requested by the eigenbehavior project, which
had re-implemented the first in Python (a union-find that was 58 % of a pair loop) and
hand-rolled the third per test file.

| | |
|---|---|
| `components(c)` | Weakly connected components of an endomap's functional graph, as a partition. Derived as a Join fold over edge partitions, not a graph traversal — the components *are* the transitive closure, and Join *is* that closure |
| `all_partitions(n)` | Every partition of `{0, ..., n-1}` |
| `bell_number(n)` | `B(n) = |Pi(U)|` for `|U| = n` |

Two ceilings, both boundary policy rather than mathematics. `all_partitions` refuses
`n > 11`: measured peak RSS is 248 MB at `n = 11` and 1,422 MB at `n = 12`, which is
where an accidental call stops being an inconvenience. `bell_number` refuses `n > 25`,
where the count stops fitting in 64 bits and would silently wrap.

`components` rejects a target outside the universe; the crate silently skips it.
`all_partitions(0)` returns the one partition of the empty universe, matching
`bell_number(0) == 1`; the crate's helper indexes an empty buffer there.

The dependency now enables the crate's `research` feature, which is where
`all_partitions` lives. It pulls in no new dependencies.

**Fixed:** the README API table still listed the five method names that 0.3.0 renamed
(`distance`, `divergence`, `jaccard`, `weighted_entropy`, `cross_entropy`). The
CHANGELOG recorded the rename and the table did not follow.

## 0.3.1

Documentation only. The README now leads with what the examples are for: finding
functional dependencies, comparing clusterings, minimising a DFA and finding connected
components use four different standard algorithms, and all four reduce to `refine` and
`coarsen`. The claim is measured rather than asserted — the logic in those examples is
9, 21, 29 and 0 lines respectively, with the rest being printing and sample data.

No code changed.

## 0.3.0 — breaking

Five methods were renamed to match the Rust crate, and four runnable examples were
added.

| was | is |
|---|---|
| `distance` | `ditset_xor_distance` |
| `jaccard` | `dit_set_jaccard` |
| `divergence` | `logical_divergence` |
| `weighted_entropy` | `logical_entropy_weighted` |
| `cross_entropy` | `cross_entropy_weighted` |

A full comparison of the two APIs found eight differences. Three are language
convention and stay — `blocks`, `ids` and `block_of` drop the `to_` and `get_` prefixes
Rust uses. The five above were arbitrary: one concept, two names, and no way to guess
which language had which. The Rust spelling wins because it says which distance, which
Jaccard, which entropy, where the shorter Python name did not.

`dit_count` was already unified in the crate's 0.3.0.

### Examples

`examples/` now holds four demos, each with a Rust twin: functional dependencies,
clustering comparison, DFA minimisation, connected components. They are run by the
test suite — an example nobody executes is unverified code that looks authoritative.

One planned selling point did not survive measurement. The clustering demo was going
to contrast `ditset_xor_distance` (a metric) against clustering indices (supposedly
not), but checking 512,000 triples showed `1 - rand_agreement` never violates the
triangle inequality either — the non-metric one is the *adjusted* Rand index, not Rand.
The demo now checks both and reports the result instead of claiming it, and the
difference it argues for is the one that holds: a scalar says how much, the lattice
says what.


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
