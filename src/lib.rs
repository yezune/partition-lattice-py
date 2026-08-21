//! Python bindings for the `partition-lattice` crate.
//!
//! The binding layer holds no lattice logic: every operation delegates to the Rust
//! crate. What it does own is the *boundary* — argument validation, error mapping,
//! and the Python data model (equality, hashing, operators).
//!
//! Two of those boundary duties exist because the Rust API is deliberately
//! permissive where a Python API should not be:
//!
//! - `Partition::from_blocks` silently drops out-of-range elements and assigns
//!   uncovered ones to block 0. Here that is a `ValueError`.
//! - Binary operations on partitions of different sizes are not meaningful; the
//!   Rust side does not check, so this layer does.

use pl::{endomap, expressiveness, incidence, pcc_arms, Partition as Inner};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Ceiling on `all_partitions`. A memory-safety bound, not a mathematical one: the
/// helper materialises every partition at once, so cost is `B(n)` and that is not a
/// gentle curve. Measured peak RSS / wall clock on this build:
///
/// | n | B(n) | peak RSS | time |
/// |---|---|---|---|
/// | 10 | 115,975 | 41 MB | 0.3 s |
/// | **11** | **678,570** | **248 MB** | **2.0 s** |
/// | 12 | 4,213,597 | 1,422 MB | 13.6 s |
///
/// 12 is where an accidental call stops being an inconvenience, so the ceiling is 11.
/// Raising it is a decision about peak RSS, not about correctness.
const MAX_ENUMERABLE: usize = 11;

/// Ceiling on `bell_number`. `B(25) = 4_638_590_332_229_999_353` fits in `u64` and
/// `B(26) = 49_631_246_523_618_756_274` does not, so past 25 the count would wrap.
const MAX_BELL: usize = 25;

/// A partition of the finite universe `{0, ..., n-1}`.
///
/// Block labels carry no information: `Partition([0, 0, 1, 1])` and
/// `Partition([7, 7, 3, 3])` are the same partition, compare equal, and hash equal.
///
/// **Order convention.** `refines` means *finer*, so `Partition.discrete(n)` is the
/// least element and `Partition.indiscrete(n)` the greatest. `meet` is the common
/// refinement and `join` the common coarsening. Much of the literature uses the
/// order-dual; when porting a formula, flip meet/join and the direction of `<=`
/// together.
#[pyclass(name = "Partition", module = "partition_lattice", frozen)]
#[derive(Clone)]
struct PyPartition {
    inner: Inner,
}

impl PyPartition {
    fn new(inner: Inner) -> Self {
        Self { inner }
    }

    /// Both operands must live over the same universe for a Meet or Join to mean
    /// anything. The Rust side does not enforce it, so the boundary does.
    fn check_same_size(&self, other: &Self) -> PyResult<()> {
        if self.inner.size() == other.inner.size() {
            Ok(())
        } else {
            Err(PyValueError::new_err(format!(
                "partitions are over different universes: {} != {}",
                self.inner.size(),
                other.inner.size()
            )))
        }
    }
}

#[pymethods]
impl PyPartition {
    /// Build from block ids: elements with equal ids share a block.
    #[new]
    fn py_new(ids: Vec<u32>) -> Self {
        Self::new(Inner::from_ids(ids))
    }

    /// The discrete partition: every element alone. The least element of the lattice.
    #[classmethod]
    fn discrete(_cls: &Bound<'_, PyType>, size: usize) -> Self {
        Self::new(Inner::discrete(size))
    }

    /// The indiscrete partition: one block. The greatest element of the lattice.
    #[classmethod]
    fn indiscrete(_cls: &Bound<'_, PyType>, size: usize) -> Self {
        Self::new(Inner::indiscrete(size))
    }

    /// Build from an explicit list of blocks over `{0, ..., size-1}`.
    ///
    /// Every element must appear in exactly one block. Unlike the Rust
    /// `from_blocks`, a partial or overlapping cover is an error rather than a
    /// silent reassignment.
    #[classmethod]
    fn from_blocks(_cls: &Bound<'_, PyType>, size: usize, blocks: Vec<Vec<usize>>) -> PyResult<Self> {
        let mut seen = vec![false; size];
        for block in &blocks {
            for &e in block {
                if e >= size {
                    return Err(PyValueError::new_err(format!(
                        "element {e} is outside the universe of size {size}"
                    )));
                }
                if seen[e] {
                    return Err(PyValueError::new_err(format!(
                        "element {e} appears in more than one block"
                    )));
                }
                seen[e] = true;
            }
        }
        if let Some(missing) = seen.iter().position(|&s| !s) {
            return Err(PyValueError::new_err(format!(
                "element {missing} is not covered by any block"
            )));
        }
        Ok(Self::new(Inner::from_blocks(size, blocks)))
    }

    // ------------------------------------------------------------ structure ---

    /// Number of blocks.
    fn block_count(&self) -> usize {
        self.inner.block_count()
    }

    /// The blocks, as sorted lists of elements.
    fn blocks(&self) -> Vec<Vec<usize>> {
        self.inner.to_blocks()
    }

    /// Canonical block ids, numbered by first appearance.
    fn ids(&self) -> Vec<u32> {
        self.inner.get_ids().to_vec()
    }

    /// The block id of one element.
    fn block_of(&self, element: usize) -> PyResult<u32> {
        self.inner
            .get_ids()
            .get(element)
            .copied()
            .ok_or_else(|| PyIndexError::new_err(format!("element {element} is out of range")))
    }

    // ----------------------------------------------------------- operations ---

    /// Common refinement: the finest partition coarser than neither operand.
    ///
    /// Named for what it does, not for its position in the lattice. `meet` and `join`
    /// name *opposite* operations in the two conventions the literature uses, so they
    /// were removed in 0.2.0 rather than left to compute the wrong thing quietly.
    fn refine(&self, other: &Self) -> PyResult<Self> {
        self.check_same_size(other)?;
        Ok(Self::new(self.inner.differentiate(&other.inner)))
    }

    /// Common coarsening: the coarsest partition finer than neither operand.
    ///
    /// The counterpart of [`Self::refine`]; see that method for why the name is what
    /// it is.
    fn coarsen(&self, other: &Self) -> PyResult<Self> {
        self.check_same_size(other)?;
        Ok(Self::new(self.inner.integrate(&other.inner)))
    }

    /// True when `self` is finer than or equal to `other`.
    fn refines(&self, other: &Self) -> PyResult<bool> {
        self.check_same_size(other)?;
        Ok(self.inner.refines(&other.inner))
    }

    // ------------------------------------------------------------- measures ---

    /// Logical entropy `h = 1 - sum (|B|/n)^2`, i.e. the fraction of ordered pairs
    /// the partition distinguishes.
    fn logical_entropy(&self) -> f64 {
        self.inner.logical_entropy()
    }

    /// The exact integer numerator of `logical_entropy`: the number of ordered
    /// pairs in different blocks. Compare these instead of the floats when you
    /// need exactness.
    fn dit_count(&self) -> usize {
        self.inner.dit_count()
    }

    /// Normalised size of the symmetric difference of the two dit sets.
    fn ditset_xor_distance(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.ditset_xor_distance(&other.inner))
    }

    /// Logical mutual information.
    fn mutual_information(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.mutual_information(&other.inner))
    }

    /// Logical divergence.
    fn logical_divergence(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.logical_divergence(&other.inner))
    }

    /// Rand agreement: the fraction of pairs the two partitions classify alike.
    fn rand_agreement(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.rand_agreement(&other.inner))
    }

    /// Jaccard similarity of the two dit sets, or `None` when neither partition
    /// distinguishes anything.
    fn dit_set_jaccard(&self, other: &Self) -> PyResult<Option<f64>> {
        self.check_same_size(other)?;
        Ok(self.inner.dit_set_jaccard(&other.inner))
    }

    /// Weighted logical entropy as an exact `(numerator, denominator)` pair.
    ///
    /// A weight is a *multiplicity*, not a probability: the result equals the
    /// ordinary logical entropy of the multiset that repeats element `u` exactly
    /// `w[u]` times. Note that a non-uniform weight breaks relabelling invariance.
    fn logical_entropy_weighted(&self, weights: Vec<u64>) -> PyResult<(u64, u64)> {
        if weights.len() != self.inner.size() {
            return Err(PyValueError::new_err(format!(
                "expected {} weights, got {}",
                self.inner.size(),
                weights.len()
            )));
        }
        Ok(self.inner.logical_entropy_weighted(&weights))
    }

    /// Logical cross-entropy: one partition measured with two product measures,
    /// as an exact `(numerator, denominator)` pair.
    fn cross_entropy_weighted(&self, p: Vec<u64>, q: Vec<u64>) -> PyResult<(u64, u64)> {
        let n = self.inner.size();
        if p.len() != n || q.len() != n {
            return Err(PyValueError::new_err(format!(
                "expected {n} weights in each argument, got {} and {}",
                p.len(),
                q.len()
            )));
        }
        Ok(self.inner.cross_entropy_weighted(&p, &q))
    }

    // -------------------------------------------------------- data model -----

    fn __len__(&self) -> usize {
        self.inner.size()
    }

    /// Only equality is defined.
    ///
    /// `<=` used to mean the refinement order, but which way that order runs is
    /// exactly what the two conventions disagree on: under Ellerman's, `a <= b` says
    /// `a` is *coarser*. A comparison that silently reverses is worse than one that
    /// does not exist, so the order operators raise `TypeError` and callers use
    /// [`Self::refines`], whose reading is the same under either convention.
    fn __richcmp__(
        &self,
        py: Python<'_>,
        other: &Self,
        op: pyo3::basic::CompareOp,
    ) -> PyObject {
        use pyo3::basic::CompareOp::*;
        let same = self.inner.size() == other.inner.size() && self.ids() == other.ids();
        match op {
            Eq => same.into_py(py),
            Ne => (!same).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __hash__(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.inner.get_ids().hash(&mut h);
        h.finish()
    }

    fn __repr__(&self) -> String {
        let blocks = self.inner.to_blocks();
        let shown: Vec<String> = blocks
            .iter()
            .take(6)
            .map(|b| format!("{{{}}}", b.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(",")))
            .collect();
        let tail = if blocks.len() > 6 { ", ..." } else { "" };
        format!(
            "Partition(n={}, blocks={}, h={:.4}) [{}{}]",
            self.inner.size(),
            blocks.len(),
            self.inner.logical_entropy(),
            shown.join(", "),
            tail
        )
    }
}

// ------------------------------------------------------------ module level ---

/// Number of ordered pairs distinguished by exactly one of the two partitions.
#[pyfunction]
fn dit_xor_count(a: &PyPartition, b: &PyPartition) -> PyResult<i64> {
    a.check_same_size(b)?;
    Ok(incidence::dit_xor_count(&a.inner, &b.inner))
}

/// Distinctions `a` makes that `b` erases. Zero exactly when `b` refines `a`.
#[pyfunction]
fn destr(a: &PyPartition, b: &PyPartition) -> PyResult<i64> {
    a.check_same_size(b)?;
    Ok(incidence::destr(&a.inner, &b.inner))
}

/// Distinctions `b` makes that `a` does not.
#[pyfunction]
fn creat(a: &PyPartition, b: &PyPartition) -> PyResult<i64> {
    a.check_same_size(b)?;
    Ok(incidence::creat(&a.inner, &b.inner))
}

// ── Endomap invariants and counting ───────────────────────────────────────────
//
// Exposed on request from the eigenbehavior project (`docs/LVM_REVIEW.md` §3.1),
// which had `components` as 58 % of a pair-loop, `all_partitions` as the unit of
// exhaustive verification, and `bell_number` hand-rolled per test file.

/// Weakly connected components of the functional graph of `c`, as a partition.
///
/// `c[i]` is the image of state `i`, so `c` is an endomap of `{0, ..., len(c)-1}`.
/// Each component holds exactly one periodic orbit, so `components(c).block_count()`
/// is also the number of periodic orbits.
///
/// The Rust helper silently skips a target outside the universe; a target that does
/// not name a state is a malformed endomap, so the boundary rejects it.
#[pyfunction]
fn components(c: Vec<usize>) -> PyResult<PyPartition> {
    let n = c.len();
    if let Some((i, &t)) = c.iter().enumerate().find(|(_, &t)| t >= n) {
        return Err(PyValueError::new_err(format!(
            "c[{i}] = {t} is not a state of a universe of size {n}"
        )));
    }
    Ok(PyPartition::new(endomap::components(&c)))
}

/// Every partition of `{0, ..., n-1}`, in no particular order.
///
/// The length is `bell_number(n)`, which grows faster than exponentially — hence the
/// ceiling at [`MAX_ENUMERABLE`], which is about peak memory rather than about what is
/// computable. `n = 0` is the empty universe, whose single partition is the empty one;
/// the Rust helper indexes an empty buffer there, so the boundary handles it.
#[pyfunction]
fn all_partitions(n: usize) -> PyResult<Vec<PyPartition>> {
    if n == 0 {
        return Ok(vec![PyPartition::new(Inner::from_ids(Vec::new()))]);
    }
    if n > MAX_ENUMERABLE {
        return Err(PyValueError::new_err(format!(
            "n = {n} would enumerate {} partitions; the ceiling is n = {MAX_ENUMERABLE}. \
             Call bell_number(n) to size the request, or work with a sample instead.",
            expressiveness::bell_number(n.min(25))
        )));
    }
    Ok(pcc_arms::all_partitions(n)
        .into_iter()
        .map(PyPartition::new)
        .collect())
}

/// `B(n) = |Pi(U)|` for `|U| = n` — the number of partitions of an n-element set.
///
/// Capped where the count stops fitting in 64 bits: `B(25)` fits and `B(26)` does not,
/// so beyond 25 the Rust helper would wrap silently rather than report.
#[pyfunction]
fn bell_number(n: usize) -> PyResult<u64> {
    if n > MAX_BELL {
        return Err(PyValueError::new_err(format!(
            "B({n}) exceeds a 64-bit integer; the ceiling is n = {MAX_BELL}"
        )));
    }
    Ok(expressiveness::bell_number(n))
}

#[pymodule]
fn _partition_lattice(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyPartition>()?;
    m.add_function(wrap_pyfunction!(dit_xor_count, m)?)?;
    m.add_function(wrap_pyfunction!(destr, m)?)?;
    m.add_function(wrap_pyfunction!(creat, m)?)?;
    m.add_function(wrap_pyfunction!(components, m)?)?;
    m.add_function(wrap_pyfunction!(all_partitions, m)?)?;
    m.add_function(wrap_pyfunction!(bell_number, m)?)?;
    Ok(())
}
