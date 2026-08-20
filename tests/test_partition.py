"""Behaviour tests for the partition_lattice Python bindings.

These are written against the documented API, not the implementation: they are the
specification the extension module has to satisfy.
"""
import math

import pytest

from partition_lattice import Partition, creat, destr, dit_xor_count


# --------------------------------------------------------------- construction ---
def test_from_ids_groups_equal_labels():
    p = Partition([0, 0, 1, 1])
    assert len(p) == 4
    assert p.block_count() == 2
    assert p.blocks() == [[0, 1], [2, 3]]


def test_labels_are_arbitrary():
    """Only the grouping matters; the label values carry no information."""
    assert Partition([0, 0, 1, 1]) == Partition([7, 7, 3, 3])
    assert Partition([0, 0, 1, 1]) != Partition([0, 1, 0, 1])


def test_discrete_and_indiscrete():
    assert Partition.discrete(4).block_count() == 4
    assert Partition.indiscrete(4).block_count() == 1
    assert Partition.discrete(4) == Partition([0, 1, 2, 3])
    assert Partition.indiscrete(4) == Partition([0, 0, 0, 0])


def test_from_blocks():
    p = Partition.from_blocks(4, [[0, 1], [2, 3]])
    assert p == Partition([0, 0, 1, 1])


def test_from_blocks_rejects_bad_input():
    with pytest.raises(ValueError):
        Partition.from_blocks(4, [[0, 1], [1, 2, 3]])  # element 1 in two blocks
    with pytest.raises(ValueError):
        Partition.from_blocks(4, [[0, 1]])             # 2 and 3 uncovered


def test_empty_partition():
    p = Partition([])
    assert len(p) == 0
    assert p.block_count() == 0
    assert p.logical_entropy() == 0.0


# ---------------------------------------------------------------- operations ---
def test_refine_is_the_common_refinement():
    a, b = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1])
    assert a.refine(b).block_count() == 4      # every element separated
    assert a.coarsen(b).block_count() == 1     # everything merged


def test_refine_and_coarsen_are_idempotent_and_commutative():
    a, b = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1])
    assert a.refine(a) == a and a.coarsen(a) == a
    assert a.refine(b) == b.refine(a)
    assert a.coarsen(b) == b.coarsen(a)


def test_absorption():
    a, b = Partition([0, 0, 1, 2]), Partition([0, 1, 1, 2])
    assert a.refine(a.coarsen(b)) == a
    assert a.coarsen(a.refine(b)) == a


def test_convention_dependent_names_are_gone():
    """`meet`/`join`/`&`/`|`/`<=` meant the opposite operation under the other
    convention, so they were removed rather than left to fail silently."""
    a, b = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1])
    for name in ("meet", "join"):
        assert not hasattr(a, name), f"{name} must be gone, not silently reinterpreted"
    for op in (lambda: a & b, lambda: a | b,
               lambda: a <= b, lambda: a < b, lambda: a >= b, lambda: a > b):
        with pytest.raises(TypeError):
            op()


def test_equality_still_works():
    # Only the *order* comparisons were convention-dependent; equality was not.
    assert Partition([0, 0, 1, 1]) == Partition([7, 7, 3, 3])
    assert Partition([0, 0, 1, 1]) != Partition([0, 1, 0, 1])


def test_size_mismatch_is_an_error():
    with pytest.raises(ValueError):
        Partition([0, 0, 1]).refine(Partition([0, 1]))


# --------------------------------------------------------------------- order ---
def test_refinement_order_finer_is_smaller():
    """`refines` means *finer*: Bottom (discrete) is the least element."""
    fine, coarse = Partition([0, 1, 2, 3]), Partition([0, 0, 1, 1])
    assert fine.refines(coarse)
    assert not coarse.refines(fine)
    assert Partition.discrete(4).refines(Partition.indiscrete(4))


def test_meet_is_below_both_and_join_above_both():
    a, b = Partition([0, 0, 1, 2]), Partition([0, 1, 1, 2])
    m, j = a.refine(b), a.coarsen(b)
    assert m.refines(a) and m.refines(b)
    assert a.refines(j) and b.refines(j)


# ------------------------------------------------------------------ measures ---
def test_logical_entropy():
    # two blocks of two out of four elements: 1 - (2/4)^2 - (2/4)^2 = 0.5
    assert Partition([0, 0, 1, 1]).logical_entropy() == 0.5
    assert Partition.indiscrete(4).logical_entropy() == 0.0
    assert Partition.discrete(4).logical_entropy() == 0.75


def test_dit_count_is_the_exact_numerator():
    p = Partition([0, 0, 1, 1])
    n = len(p)
    assert p.dit_count() == 8
    assert p.dit_count() / (n * n) == p.logical_entropy()


def test_entropy_is_monotone_under_refinement():
    fine, coarse = Partition([0, 1, 2, 3]), Partition([0, 0, 1, 1])
    assert fine.logical_entropy() > coarse.logical_entropy()


def test_distance_is_a_metric_on_small_cases():
    a, b, c = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1]), Partition([0, 1, 2, 3])
    assert a.ditset_xor_distance(a) == 0.0
    assert a.ditset_xor_distance(b) == b.ditset_xor_distance(a)
    assert a.ditset_xor_distance(c) <= a.ditset_xor_distance(b) + b.ditset_xor_distance(c) + 1e-12


def test_mutual_information_and_divergence():
    a, b = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1])
    assert math.isclose(a.mutual_information(a), a.logical_entropy())
    assert a.logical_divergence(a) == 0.0


def test_weighted_entropy_matches_multiset_expansion():
    """A weight is a multiplicity: h_w(pi) == h of the expanded multiset."""
    p = Partition([0, 0, 1])
    num, den = p.logical_entropy_weighted([2, 1, 1])
    expanded = Partition([0, 0, 0, 1])  # element 0 repeated twice
    assert num / den == expanded.logical_entropy()


def test_weighted_entropy_with_unit_weights_matches_plain():
    p = Partition([0, 0, 1, 1])
    num, den = p.logical_entropy_weighted([1, 1, 1, 1])
    assert num / den == p.logical_entropy()


# ------------------------------------------------------- module-level counts ---
def test_dit_xor_count_and_destr_creat():
    a, b = Partition([0, 0, 1, 1]), Partition([0, 1, 0, 1])
    assert dit_xor_count(a, b) == destr(a, b) + creat(a, b)
    assert destr(a, a) == 0 and creat(a, a) == 0
    # destr(a, b) == 0 exactly when b refines a
    assert destr(a, Partition.discrete(4)) == 0


# ------------------------------------------------- convention-neutral names ---
def test_refine_is_finer_and_coarsen_is_coarser():
    a, b = Partition([0, 0, 1, 2]), Partition([0, 1, 1, 2])
    assert a.refine(b).refines(a) and a.refine(b).refines(b)
    assert a.refines(a.coarsen(b)) and b.refines(a.coarsen(b))


def test_refine_raises_entropy_and_coarsen_lowers_it():
    # The one asymmetry no renaming can hide: h peaks at the discrete partition.
    a, b = Partition([0, 0, 1, 2]), Partition([0, 1, 1, 2])
    assert a.refine(b).logical_entropy() >= max(a.logical_entropy(), b.logical_entropy())
    assert a.coarsen(b).logical_entropy() <= min(a.logical_entropy(), b.logical_entropy())


def test_coarsen_rejects_size_mismatch():
    with pytest.raises(ValueError):
        Partition([0, 0, 1]).coarsen(Partition([0, 1]))


# ------------------------------------------------------------------- dunders ---
def test_repr_and_hash():
    p = Partition([0, 0, 1, 1])
    assert "Partition" in repr(p)
    assert hash(p) == hash(Partition([5, 5, 9, 9]))
    assert len({p, Partition([5, 5, 9, 9])}) == 1


def test_ids_roundtrip():
    p = Partition([3, 3, 7, 1])
    assert Partition(p.ids()) == p
