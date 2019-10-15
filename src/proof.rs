use crate::{Poseidon, PoseidonLeaf, Scalar, MERKLE_ARITY, MERKLE_HEIGHT};
use std::ops;

/// Set of [`ProofElement`] defining a path to the root of the tree.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Proof<T: PoseidonLeaf> {
    pos: usize,
    data: [(usize, [Option<T>; MERKLE_ARITY]); MERKLE_HEIGHT - 1],
}

impl<T: PoseidonLeaf> Default for Proof<T> {
    fn default() -> Self {
        Proof {
            pos: 0,
            data: [(0, [None; MERKLE_ARITY]); MERKLE_HEIGHT - 1],
        }
    }
}

impl<T: PoseidonLeaf> Proof<T> {
    pub(crate) fn push(&mut self, idx: usize, leaves: &[Option<T>]) {
        let (i, proof) = &mut self.data[self.pos];

        proof.copy_from_slice(leaves);
        *i = idx;

        self.pos += 1;
    }

    /// Return the raw proof data
    pub fn data(&self) -> &[(usize, [Option<T>; MERKLE_ARITY]); MERKLE_HEIGHT - 1] {
        &self.data
    }

    /// Verify if the provided leaf corresponds to the proof in the merkle construction
    pub fn verify(&self, leaf: &T, root: &T) -> bool
    where
        Scalar: ops::Mul<T, Output = T>,
    {
        let mut leaf = *leaf;
        let mut h = Poseidon::default();

        for i in 0..self.data.len() {
            let (idx, data) = self.data[i];

            h.replace(&data[0..MERKLE_ARITY]);
            h.insert_unchecked(idx, leaf);

            leaf = h.hash();
        }

        &leaf == root
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn proof_verify() {
        let mut t = MerkleTree::<Scalar>::default();
        for i in 0..MERKLE_WIDTH {
            t.insert_unchecked(i, Scalar::from(i as u64));
        }

        let root = t.root();
        let i = MERKLE_WIDTH / 3;

        let proof = t.proof_index(i);
        assert!(proof.verify(&Scalar::from(i as u64), &root));
    }

    #[test]
    fn proof_verify_failure() {
        let mut t = MerkleTree::<Scalar>::default();
        for i in 0..MERKLE_WIDTH {
            t.insert_unchecked(i, Scalar::from(i as u64));
        }

        let root = t.root();
        let i = MERKLE_WIDTH / 3;

        let proof = t.proof_index(i + 1);
        assert!(!proof.verify(&Scalar::from(i as u64), &root));
    }
}
