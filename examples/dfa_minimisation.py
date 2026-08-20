"""Minimising a DFA -- which is partition refinement and nothing else.

Moore's algorithm: separate accepting from non-accepting states, then repeatedly split
any block whose members disagree about *which block* their transitions land in. Stop
when a round changes nothing; the blocks of the fixed point are the minimal DFA's
states.

Each round is one ``refine``, and "changed nothing" is an exact integer comparison of
``dit_count`` -- no epsilon, no ambiguity about reaching the fixed point.

    python examples/dfa_minimisation.py
"""

from partition_lattice import Partition

#          a  b
DELTA = [[1, 2], [3, 4], [4, 3], [5, 5], [5, 5], [5, 5]]
ACCEPTING = [False, False, False, True, True, True]


def signature_partition(current):
    """Split states by where their transitions land under the current partition."""
    ids = current.ids()
    seen = {}
    sig = []
    for targets in DELTA:
        key = tuple(ids[t] for t in targets)
        sig.append(seen.setdefault(key, len(seen)))
    return Partition(sig)


def minimise():
    current = Partition([int(a) for a in ACCEPTING])
    rounds = 0
    while True:
        nxt = current.refine(signature_partition(current))
        rounds += 1
        if nxt.dit_count() == current.dit_count():
            return nxt, rounds
        current = nxt


def main():
    print(f"DFA with {len(DELTA)} states")
    minimal, rounds = minimise()
    print(f"fixed point reached in {rounds} rounds\n")

    print(f"equivalence classes ({minimal.block_count()} states in the minimal DFA)")
    for i, block in enumerate(minimal.blocks()):
        acc = "  accepting" if ACCEPTING[block[0]] else ""
        print(f"  q{i}  {{{','.join('s' + str(s) for s in block)}}}{acc}")

    print("\nthe refinement chain")
    p = Partition([int(a) for a in ACCEPTING])
    round_no = 0
    while True:
        print(f"  round {round_no}: {p.block_count()} blocks, {p.dit_count()} distinctions")
        nxt = p.refine(signature_partition(p))
        if nxt.dit_count() == p.dit_count():
            break
        assert nxt.refines(p), "refinement must move down the lattice"
        p, round_no = nxt, round_no + 1
    print("  (each round refines the previous, so termination is the lattice being finite)")


if __name__ == "__main__":
    main()
