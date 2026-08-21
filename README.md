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
| Measures | `logical_entropy()`, `dit_count()`, `ditset_xor_distance(o)`, `mutual_information(o)`, `logical_divergence(o)`, `rand_agreement(o)`, `dit_set_jaccard(o)` |
| Exact measures | `logical_entropy_weighted(w)`, `cross_entropy_weighted(p, q)` |
| Module | `dit_xor_count(a, b)`, `destr(a, b)`, `creat(a, b)` |
| Endomaps | `components(c)` |
| Counting | `all_partitions(n)`, `bell_number(n)` |

Unlike the Rust `from_blocks`, `Partition.from_blocks` rejects a partial or
overlapping cover instead of silently reassigning elements. Three other places where
this layer is stricter than the crate: `components` rejects a target outside the
universe, `all_partitions` refuses `n > 11` rather than exhaust memory, and
`bell_number` refuses `n > 25` rather than wrap a 64-bit counter.

`components(c)` takes an endomap as a list — `c[i]` is the image of state `i` — and
returns the partition into weakly connected components of its functional graph. Each
component holds exactly one periodic orbit, so `components(c).block_count()` counts
those orbits.

## Examples — four algorithms, two operations

Finding functional dependencies, comparing clusterings, minimising a DFA, and finding
connected components are four different problems with four different standard
algorithms. All four turn out to be the same two lattice operations:

| problem | usually reached for | what it is here |
|---|---|---|
| functional dependencies | TANE and relatives | `partition(X).refines(partition(Y))` |
| comparing two clusterings | Rand, ARI, NMI | `coarsen` for the consensus, `destr` / `creat` for the direction |
| DFA minimisation | Hopcroft, Moore | `refine` until `dit_count` stops growing |
| connected components | union-find | `coarsen` folded over the edges |

That is the case for the library. Not that it computes these faster, but that it is
the structure those algorithms were reconstructing each time. Once the data is a
partition, the parts that normally need care — fixed points, transitive closure,
composite keys — come from the lattice rather than from you, and what is left is
small: the logic in these examples is 9 lines for components, 21 for dependencies and
29 for DFA minimisation, with the clustering one containing no algorithm at all, only
calls. The rest of each file is printing and sample data.

Two properties do the work throughout:

- **Counts stay integers.** `dit_count` is the exact numerator of the entropy, so a
  fixed point is `==` rather than `abs(a - b) < eps`, and near-misses rank without
  float ties deciding the winner.
- **The order is available, not just a number.** A similarity index answers *how much*.
  `refine` and `coarsen` answer *what*: which distinctions two groupings agree on,
  which one of them drew and the other erased.

Run them as-is with `python examples/<name>.py`. Each has a Rust twin in the
[crate](https://github.com/yezune/partition-lattice/tree/main/examples).

| | what it shows |
|---|---|
| `functional_dependencies` | `X -> Y` **is** the refinement order, so finding dependencies needs no dependency-checking algorithm. Composite determinants are one `refine`. |
| `clustering_comparison` | A scalar index says *how much* two clusterings differ; `coarsen` / `refine` / `destr` / `creat` say *in what way*, and keep the direction a single number discards. |
| `dfa_minimisation` | Moore's algorithm is `refine` in a loop, and its fixed point is an exact integer comparison — no epsilon. |
| `graph_components` | Folding edges with `coarsen` gives connected components. The join is the union-find. |

## Install

```
pip install partition-lattice
```

Wheels are built against the stable ABI (`abi3`), so one wheel per platform covers
CPython 3.8 and newer.

## License

MIT OR Apache-2.0, at your option.
