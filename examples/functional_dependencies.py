"""Finding functional dependencies in a table, using nothing but the lattice order.

A functional dependency ``X -> Y`` says rows agreeing on ``X`` agree on ``Y``. Group the
rows by ``X``, group them again by ``Y``, and the dependency holds exactly when the
first grouping is *finer* than the second::

    X -> Y   iff   partition(X).refines(partition(Y))

That is not an analogy, it is the definition restated -- which is why there is no
dependency-checking algorithm here. Composite determinants come for free, since
``partition({A, B})`` is ``partition(A).refine(partition(B))``.

    python examples/functional_dependencies.py
"""

from functools import reduce

from partition_lattice import Partition

COLUMNS = ["zip", "city", "country", "currency", "population"]
ROWS = [
    ["01", "Seoul", "KR", "KRW", "9.4M"],
    ["02", "Seoul", "KR", "KRW", "9.4M"],
    ["03", "Busan", "KR", "KRW", "3.3M"],
    ["04", "Tokyo", "JP", "JPY", "14M"],
    ["05", "Tokyo", "JP", "JPY", "14M"],
    ["06", "Osaka", "JP", "JPY", "2.7M"],
]


def partition_of(col):
    """Group rows by the value in one column; equal values share a block."""
    seen = {}
    return Partition([seen.setdefault(r[col], len(seen)) for r in ROWS])


def partition_of_set(cols):
    """The partition induced by several columns: their common refinement."""
    return reduce(lambda a, b: a.refine(b), (partition_of(c) for c in cols))


def violation(lhs, rhs):
    """How badly ``X -> Y`` fails, as an exact count of disagreeing ordered pairs.

    Zero means the dependency holds. A positive count ranks near-misses without a
    float tie deciding which candidate wins.
    """
    return lhs.refine(rhs).dit_count() - lhs.dit_count()


def main():
    print(f"table: {len(ROWS)} rows x {len(COLUMNS)} columns\n")

    print("single-column dependencies")
    for i, lhs_name in enumerate(COLUMNS):
        for j, rhs_name in enumerate(COLUMNS):
            if i == j:
                continue
            lhs, rhs = partition_of(i), partition_of(j)
            if lhs.refines(rhs):
                print(f"  holds  {lhs_name:<10} -> {rhs_name}")
            elif violation(lhs, rhs) <= 4:
                print(f"  fails  {lhs_name:<10} -> {rhs_name:<10} ({violation(lhs, rhs)} row-pairs disagree)")

    print("\ncomposite determinant")
    cols = [1, 4]  # city, population
    lhs = partition_of_set(cols)
    name = "{" + ", ".join(COLUMNS[c] for c in cols) + "}"
    for j in (2, 3):
        verdict = "holds " if lhs.refines(partition_of(j)) else "fails "
        print(f"  {verdict} {name} -> {COLUMNS[j]}")

    print("\nkeys (partition is discrete, so it determines every column)")
    discrete = Partition.discrete(len(ROWS))
    for i, name in enumerate(COLUMNS):
        if partition_of(i).dit_count() == discrete.dit_count():
            print(f"  {name} is a key")

    print("\nranking near-misses by exact violation count")
    near = [
        (COLUMNS[i], COLUMNS[j], violation(partition_of(i), partition_of(j)))
        for i in range(len(COLUMNS))
        for j in range(len(COLUMNS))
        if i != j and violation(partition_of(i), partition_of(j)) > 0
    ]
    for lhs_name, rhs_name, v in sorted(near, key=lambda t: t[2])[:5]:
        print(f"  {lhs_name:<10} -> {rhs_name:<10} {v} disagreeing row-pairs")


if __name__ == "__main__":
    main()
