"""Connected components -- built by coarsening, one edge at a time.

An edge says "these two vertices belong together": a partition with everything alone
except one merged pair. Fold those with ``coarsen`` and you have the components,
because the least upper bound of "u ~ v" facts is their transitive closure.

No union-find is written here. The join *is* the union-find.

    python examples/graph_components.py
"""

from functools import reduce

from partition_lattice import Partition

N = 8
EDGES = [(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (6, 7)]


def edge(u, v):
    """The partition asserting one edge and nothing else."""
    return Partition([u if i == v else i for i in range(N)])


def components(edges):
    return reduce(lambda acc, e: acc.coarsen(edge(*e)), edges, Partition.discrete(N))


def show(label, p):
    blocks = " ".join("{" + ",".join(map(str, b)) + "}" for b in p.blocks())
    print(f"  {label:<18} {p.block_count()} components  {blocks}")


def main():
    print(f"graph: {N} vertices, {len(EDGES)} edges")
    comp = components(EDGES)
    show("components", comp)

    print("\nadding edges one at a time (each step is coarser than the last)")
    acc = Partition.discrete(N)
    for e in EDGES:
        nxt = acc.coarsen(edge(*e))
        assert acc.refines(nxt), "adding an edge cannot split a component"
        print(f"  +{e}  {nxt.block_count()} components, {nxt.dit_count()} distinctions remaining")
        acc = nxt

    print("\nredundant edges (adding them merges nothing new)")
    acc = Partition.discrete(N)
    for e in EDGES:
        nxt = acc.coarsen(edge(*e))
        if nxt.dit_count() == acc.dit_count():
            print(f"  {e} is redundant -- its endpoints were already connected")
        acc = nxt

    other = components([(0, 1), (1, 2), (2, 0), (3, 4), (5, 6), (6, 7)])
    print("\ncomparing two graphs by component structure")
    show("graph 1", comp)
    show("graph 2", other)
    print(f"  d_xor between them: {comp.ditset_xor_distance(other):.4f}")
    show("agreed", comp.coarsen(other))


if __name__ == "__main__":
    main()
