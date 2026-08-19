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

use pl::{incidence, Partition as Inner};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

    /// Common refinement (greatest lower bound).
    fn meet(&self, other: &Self) -> PyResult<Self> {
        self.check_same_size(other)?;
        Ok(Self::new(self.inner.differentiate(&other.inner)))
    }

    /// Common coarsening (least upper bound).
    fn join(&self, other: &Self) -> PyResult<Self> {
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
        self.inner.distinction_count()
    }

    /// Normalised size of the symmetric difference of the two dit sets.
    fn distance(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.ditset_xor_distance(&other.inner))
    }

    /// Logical mutual information.
    fn mutual_information(&self, other: &Self) -> PyResult<f64> {
        self.check_same_size(other)?;
        Ok(self.inner.mutual_information(&other.inner))
    }

    /// Logical divergence.
    fn divergence(&self, other: &Self) -> PyResult<f64> {
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
    fn jaccard(&self, other: &Self) -> PyResult<Option<f64>> {
        self.check_same_size(other)?;
        Ok(self.inner.dit_set_jaccard(&other.inner))
    }

    /// Weighted logical entropy as an exact `(numerator, denominator)` pair.
    ///
    /// A weight is a *multiplicity*, not a probability: the result equals the
    /// ordinary logical entropy of the multiset that repeats element `u` exactly
    /// `w[u]` times. Note that a non-uniform weight breaks relabelling invariance.
    fn weighted_entropy(&self, weights: Vec<u64>) -> PyResult<(u64, u64)> {
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
    fn cross_entropy(&self, p: Vec<u64>, q: Vec<u64>) -> PyResult<(u64, u64)> {
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

    fn __and__(&self, other: &Self) -> PyResult<Self> {
        self.meet(other)
    }

    fn __or__(&self, other: &Self) -> PyResult<Self> {
        self.join(other)
    }

    fn __richcmp__(&self, other: &Self, op: pyo3::basic::CompareOp) -> PyResult<bool> {
        use pyo3::basic::CompareOp::*;
        let same = self.inner.size() == other.inner.size() && self.ids() == other.ids();
        Ok(match op {
            Eq => same,
            Ne => !same,
            // The order is the refinement order, and it is only partial: `a <= b`
            // being false does not make `a > b` true.
            Le => self.refines(other)?,
            Lt => !same && self.refines(other)?,
            Ge => other.refines(self)?,
            Gt => !same && other.refines(self)?,
        })
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

#[pymodule]
fn _partition_lattice(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyPartition>()?;
    m.add_function(wrap_pyfunction!(dit_xor_count, m)?)?;
    m.add_function(wrap_pyfunction!(destr, m)?)?;
    m.add_function(wrap_pyfunction!(creat, m)?)?;
    Ok(())
}
