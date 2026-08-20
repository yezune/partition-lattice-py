"""Comparing two clusterings -- and getting more back than a number.

Every clustering-comparison index answers *how much* two groupings differ. None
answers *in what way*. The lattice does, because the two clusterings have a meet and a
join::

    a.coarsen(b)   the finest grouping both agree on -- the consensus
    a.refine(b)    the coarsest grouping respecting every split either made
    destr(a, b)    distinctions a draws that b erases
    creat(a, b)    distinctions b draws that a does not

The last two matter because a scalar conflates them: two comparisons with the same
distance can be "a split what b merged" or the reverse, which call for opposite fixes.

    python examples/clustering_comparison.py

On metricity: ``ditset_xor_distance`` satisfies the triangle inequality, and so does
``1 - rand_agreement``. This script checks both rather than asserting either.
"""

import random
from functools import reduce

from partition_lattice import Partition, creat, destr, dit_xor_count


def show(label, p):
    blocks = " ".join("{" + ",".join(map(str, b)) + "}" for b in p.blocks())
    print(f"  {label:<12} {p.block_count()} blocks  {blocks}")


def main():
    a = Partition([0, 0, 0, 0, 1, 1, 1, 2, 2, 2])
    b = Partition([0, 0, 0, 3, 1, 1, 1, 1, 1, 1])

    print("two clusterings of the same 10 points")
    show("a", a)
    show("b", b)

    print("\nhow much they differ")
    print(f"  d_xor          {a.ditset_xor_distance(b):.4f}")
    print(f"  rand agreement {a.rand_agreement(b):.4f}")
    print(f"  jaccard        {a.dit_set_jaccard(b):.4f}")

    print("\nin what way")
    show("consensus", a.coarsen(b))
    show("combined", a.refine(b))
    print(f"  a splits what b merges: {destr(a, b)} ordered pairs")
    print(f"  b splits what a merges: {creat(a, b)} ordered pairs")
    print("  (a scalar distance adds these two together and loses the direction)")

    runs = [
        Partition([0, 0, 0, 1, 1, 1, 2, 2, 2, 2]),
        Partition([0, 0, 0, 1, 1, 2, 2, 2, 2, 2]),
        Partition([0, 0, 0, 1, 1, 1, 1, 2, 2, 2]),
    ]
    consensus = reduce(lambda x, y: x.coarsen(y), runs)
    strict = reduce(lambda x, y: x.refine(y), runs)
    print("\nconsensus over 3 runs")
    show("agreed", consensus)
    show("any-split", strict)
    print(f"  stability: {consensus.logical_entropy() / strict.logical_entropy():.4f}"
          " of possible distinctions survive all three")

    print("\nranking runs against a reference, by exact integer")
    reference = Partition([0, 0, 0, 1, 1, 1, 2, 2, 2, 2])
    for i, d in sorted(((i, dit_xor_count(reference, r)) for i, r in enumerate(runs)),
                       key=lambda t: t[1]):
        print(f"  run {i}: {d} differing ordered pairs")

    rng = random.Random(0xDEADBEEF)
    ps = [Partition([rng.randrange(rng.choice([2, 3, 4, 5])) for _ in range(10)])
          for _ in range(25)]
    dv = rv = checked = 0
    for x in ps:
        for y in ps:
            for z in ps:
                checked += 1
                if x.ditset_xor_distance(z) > x.ditset_xor_distance(y) + y.ditset_xor_distance(z) + 1e-12:
                    dv += 1
                r = lambda p, q: 1.0 - p.rand_agreement(q)  # noqa: E731
                if r(x, z) > r(x, y) + r(y, z) + 1e-12:
                    rv += 1
    print(f"\ntriangle inequality over {checked} triples")
    print(f"  d_xor      violations: {dv}")
    print(f"  1 - rand   violations: {rv}")
    print("  (both hold; metricity is not what distinguishes them -- the algebra is)")


if __name__ == "__main__":
    main()
