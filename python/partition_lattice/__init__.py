"""Partition lattice algebra in exact integer arithmetic.

Everything here is implemented in Rust; this module only re-exports it so that the
package can carry type stubs.

See the class documentation for the two conventions that matter: `refines` means
*finer*, and `meet` is the common refinement (the order-dual of the convention used
in much of the literature).
"""

from ._partition_lattice import (
    Partition,
    __version__,
    creat,
    destr,
    dit_xor_count,
)

__all__ = ["Partition", "creat", "destr", "dit_xor_count", "__version__"]
