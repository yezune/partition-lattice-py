"""The examples are executable documentation, so they get executed.

An example that is never run is unverified code that looks authoritative. These tests
run each one and check the claim it is built around, so a rename or a semantic change
breaks the example loudly instead of leaving a wrong story in the repository.
"""

import pathlib
import runpy
import subprocess
import sys

import pytest

EXAMPLES = sorted((pathlib.Path(__file__).parent.parent / "examples").glob("*.py"))


@pytest.mark.parametrize("path", EXAMPLES, ids=lambda p: p.stem)
def test_example_runs(path):
    r = subprocess.run([sys.executable, str(path)], capture_output=True, text=True)
    assert r.returncode == 0, f"{path.name} failed:\n{r.stderr}"
    assert r.stdout.strip(), f"{path.name} printed nothing"


def test_there_are_examples():
    # Guards against the glob silently matching nothing, which would make every
    # parametrised test above vacuous.
    assert len(EXAMPLES) >= 4, f"expected the four demos, found {[p.name for p in EXAMPLES]}"


# --- the claim each example is built around, checked directly -------------------

def test_functional_dependency_is_the_refinement_order():
    from partition_lattice import Partition
    zip_ = Partition([0, 1, 2, 3, 4, 5])
    city = Partition([0, 0, 1, 2, 2, 3])
    assert zip_.refines(city)          # zip determines city
    assert not city.refines(zip_)      # city does not determine zip
    # and the violation count is exact: Seoul {0,1} and Tokyo {3,4} each contribute
    # two ordered pairs
    assert city.refine(zip_).dit_count() - city.dit_count() == 4


def test_scalar_loses_the_direction_that_destr_creat_keeps():
    from partition_lattice import Partition, creat, destr, dit_xor_count
    a = Partition([0, 0, 0, 0, 1, 1, 1, 2, 2, 2])
    b = Partition([0, 0, 0, 3, 1, 1, 1, 1, 1, 1])
    # The scalar is the sum; the two components are different sizes, and that
    # asymmetry is what a single number throws away.
    assert destr(a, b) + creat(a, b) == dit_xor_count(a, b)
    assert destr(a, b) != creat(a, b)


def test_dfa_minimisation_reaches_a_fixed_point():
    mod = runpy.run_path(str(pathlib.Path(__file__).parent.parent / "examples" / "dfa_minimisation.py"))
    minimal, rounds = mod["minimise"]()
    assert minimal.block_count() == 3, "the 6-state DFA collapses to 3"
    assert rounds >= 2


def test_components_never_split_as_edges_arrive():
    from partition_lattice import Partition
    n = 6
    def edge(u, v):
        return Partition([u if i == v else i for i in range(n)])
    acc = Partition.discrete(n)
    for u, v in [(0, 1), (1, 2), (3, 4)]:
        nxt = acc.coarsen(edge(u, v))
        assert acc.refines(nxt), "adding an edge must only coarsen"
        acc = nxt
    assert acc.block_count() == 3  # {0,1,2} {3,4} {5}
