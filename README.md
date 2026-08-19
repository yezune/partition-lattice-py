# partition-lattice

Partition lattice algebra in exact integer arithmetic: **meet, join, and logical
entropy**. Python bindings for the Rust crate
[`partition-lattice`](https://crates.io/crates/partition-lattice).

A *partition* of a finite universe splits it into disjoint blocks. Partitions form a
lattice under refinement, and this package exposes that lattice directly — the two
operations, the order, and the measures — without going through probabilities or
floats where integers will do.

```python
from partition_lattice import Partition

a = Partition([0, 0, 1, 1])   # {0,1}{2,3}
b = Partition([0, 1, 0, 1])   # {0,2}{1,3}

a.refine(b).block_count()     # 4 — every element separated
a.coarsen(b).block_count()    # 1 — everything merged
a.logical_entropy()           # 0.5
a.dit_count()                 # 8 — the exact integer numerator
```

Block labels carry no information, so equal groupings are equal partitions:

```python
Partition([0, 0, 1, 1]) == Partition([7, 7, 3, 3])   # True
```

## Two conventions worth knowing before you start

**Order.** `a.refines(b)` means *finer*, so `Partition.discrete(n)` is the least
element and `Partition.indiscrete(n)` the greatest. `<=` is that refinement order.

**The operations are `refine` and `coarsen`, not `meet` and `join`.** Much of the
literature — including Ellerman's papers — calls the common refinement *join*; other
sources call it *meet*. Both are standard, and they are opposite conventions on the
same structure, so a name like `meet` cannot be read without first knowing which
convention is in force. `refine` and `coarsen` say what the operation does, so they
read the same either way.

`meet`, `join`, `&`, `|` and the order comparisons `<= < >= >` were **removed in
0.2.0**. They would have kept working while meaning the opposite thing to a reader
coming from the other convention — a silent wrong answer. They now raise, and
`refines()` covers the order:

```python
a.refines(b)     # is a finer than b?  (same reading under either convention)
a == b           # equality was never convention-dependent, and is unchanged
```

## Exactness

Distinction counts are integers and stay integers. `logical_entropy()` is the only
place a division happens, and its numerator is available separately as
`dit_count()`, so entropies can be compared exactly rather than through floats.

`weighted_entropy(weights)` and `cross_entropy(p, q)` return exact
`(numerator, denominator)` pairs. A weight is a *multiplicity*, not a probability:
the result equals the ordinary logical entropy of the multiset repeating element `u`
exactly `w[u]` times. Note that a non-uniform weight breaks relabelling invariance.

## API

| | |
|---|---|
| Construction | `Partition(ids)`, `Partition.discrete(n)`, `Partition.indiscrete(n)`, `Partition.from_blocks(n, blocks)` |
| Structure | `block_count()`, `blocks()`, `ids()`, `block_of(e)`, `len(p)` |
| Operations | `refine(o)`, `coarsen(o)`, `refines(o)` |
| Measures | `logical_entropy()`, `dit_count()`, `distance(o)`, `mutual_information(o)`, `divergence(o)`, `rand_agreement(o)`, `jaccard(o)` |
| Exact measures | `weighted_entropy(w)`, `cross_entropy(p, q)` |
| Module | `dit_xor_count(a, b)`, `destr(a, b)`, `creat(a, b)` |

Unlike the Rust `from_blocks`, `Partition.from_blocks` rejects a partial or
overlapping cover instead of silently reassigning elements.

## Install

```
pip install partition-lattice
```

Wheels are built against the stable ABI (`abi3`), so one wheel per platform covers
CPython 3.8 and newer.

## License

MIT OR Apache-2.0, at your option.
