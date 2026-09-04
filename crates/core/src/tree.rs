//! Member set as a Merkle tree over nullifier public keys.
//!
//! The shape deliberately mirrors LEZ's commitment-set tree
//! (`compute_digest_for_path`, `lee/state_machine/core/src/commitment.rs`): a leaf is hashed, then
//! combined pairwise with `SHA256(left ‖ right)`, choosing sides by the index bit and shifting right
//! one level at a time. Matching LEZ means one mental model for both trees, and the guest-side path
//! walk is the same code shape reviewers already know from upstream.
//!
//! Only [`root_from_path`] runs inside the guest. Tree construction is host-only: members build the
//! tree once when the multisig is created.

use crate::{member_leaf, sha256_pair, Digest32};

/// A member's position in the tree plus the sibling hashes from leaf to root.
///
/// Mirrors LEZ's `MembershipProof = (usize, Vec<[u8; 32]>)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberPath {
    /// Leaf index, least-significant bit first when walking up.
    pub index: usize,
    /// Sibling hash at each level, ordered leaf-to-root.
    pub siblings: Vec<Digest32>,
}

/// Recomputes the root a `(leaf, index, siblings)` triple implies.
///
/// This is the guest-side check: the caller compares the result against the `member_root` recorded
/// in the multisig config account. A path that does not belong to the set yields a different root.
#[must_use]
pub fn root_from_path(leaf: &Digest32, path: &MemberPath) -> Digest32 {
    let mut node = *leaf;
    let mut index = path.index;
    for sibling in &path.siblings {
        node = if index & 1 == 0 {
            sha256_pair(&node, sibling)
        } else {
            sha256_pair(sibling, &node)
        };
        index >>= 1;
    }
    node
}

/// A member set, built host-side from the members' nullifier public keys.
///
/// The tree is padded to a power of two by repeating the last leaf, so every member has a path of
/// uniform depth. Padding with a duplicate rather than a zero leaf avoids introducing a
/// "zero member" whose preimage is known.
#[derive(Debug, Clone)]
pub struct MemberTree {
    leaves: Vec<Digest32>,
    levels: Vec<Vec<Digest32>>,
}

impl MemberTree {
    /// Builds the tree from member nullifier public keys, in the given order.
    ///
    /// Returns `None` for an empty member set — a multisig with no members is not a multisig, and
    /// the caller gets a `None` to handle rather than a panic.
    #[must_use]
    pub fn new(npks: &[Digest32]) -> Option<Self> {
        if npks.is_empty() {
            return None;
        }
        let leaves: Vec<Digest32> = npks.iter().map(member_leaf).collect();

        let mut padded = leaves.clone();
        // Repeat the last leaf up to the next power of two.
        if let Some(&last) = padded.last() {
            while !padded.len().is_power_of_two() {
                padded.push(last);
            }
        }

        let mut levels = vec![padded];
        while levels.last().map_or(0, Vec::len) > 1 {
            let current = levels.last()?;
            let next: Vec<Digest32> = current
                .chunks(2)
                .filter_map(|pair| match pair {
                    [a, b] => Some(sha256_pair(a, b)),
                    [a] => Some(sha256_pair(a, a)),
                    _ => None,
                })
                .collect();
            levels.push(next);
        }

        Some(Self { leaves, levels })
    }

    /// The Merkle root committed to by `config_hash`.
    #[must_use]
    pub fn root(&self) -> Digest32 {
        self.levels
            .last()
            .and_then(|level| level.first())
            .copied()
            .unwrap_or([0_u8; 32])
    }

    /// Number of real members, excluding padding.
    #[must_use]
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Whether the set has no members. Always `false` — [`MemberTree::new`] rejects an empty set —
    /// but present because clippy asks for it alongside [`MemberTree::len`].
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// The authentication path for the member at `index`.
    ///
    /// Returns `None` if `index` is not a real member.
    #[must_use]
    pub fn path(&self, index: usize) -> Option<MemberPath> {
        if index >= self.leaves.len() {
            return None;
        }
        let mut siblings = Vec::new();
        let mut idx = index;
        // Every level except the root contributes one sibling.
        for level in self.levels.iter().take(self.levels.len().saturating_sub(1)) {
            let sibling_idx = idx ^ 1;
            let sibling = level.get(sibling_idx).or_else(|| level.get(idx))?;
            siblings.push(*sibling);
            idx >>= 1;
        }
        Some(MemberPath { index, siblings })
    }

    /// The leaf hash for the member at `index`.
    #[must_use]
    pub fn leaf(&self, index: usize) -> Option<Digest32> {
        self.leaves.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn npks(n: u8) -> Vec<Digest32> {
        (0..n).map(|i| [i.wrapping_add(1); 32]).collect()
    }

    #[test]
    fn empty_member_set_is_rejected() {
        assert!(MemberTree::new(&[]).is_none());
    }

    #[test]
    fn every_member_path_reproduces_the_root() {
        // Includes non-powers-of-two, which is where padding bugs live.
        for n in 1..=9_u8 {
            let tree = MemberTree::new(&npks(n)).expect("non-empty");
            let root = tree.root();
            for i in 0..usize::from(n) {
                let leaf = tree.leaf(i).expect("real member");
                let path = tree.path(i).expect("real member");
                assert_eq!(
                    root_from_path(&leaf, &path),
                    root,
                    "member {i} of {n} failed to reproduce the root"
                );
            }
        }
    }

    #[test]
    fn a_non_member_cannot_reach_the_root() {
        let tree = MemberTree::new(&npks(3)).expect("non-empty");
        let path = tree.path(0).expect("real member");
        let outsider = member_leaf(&[0xFE; 32]);
        assert_ne!(root_from_path(&outsider, &path), tree.root());
    }

    #[test]
    fn a_tampered_sibling_cannot_reach_the_root() {
        let tree = MemberTree::new(&npks(4)).expect("non-empty");
        let leaf = tree.leaf(2).expect("real member");
        let mut path = tree.path(2).expect("real member");
        if let Some(first) = path.siblings.first_mut() {
            first[0] ^= 0xFF;
        }
        assert_ne!(root_from_path(&leaf, &path), tree.root());
    }

    #[test]
    fn a_member_cannot_reuse_another_members_path() {
        let tree = MemberTree::new(&npks(4)).expect("non-empty");
        let leaf = tree.leaf(1).expect("real member");
        let other = tree.path(2).expect("real member");
        assert_ne!(root_from_path(&leaf, &other), tree.root());
    }

    #[test]
    fn out_of_range_index_has_no_path() {
        let tree = MemberTree::new(&npks(3)).expect("non-empty");
        assert!(tree.path(3).is_none(), "padding is not a member");
        assert!(tree.leaf(3).is_none());
    }

    #[test]
    fn member_order_changes_the_root() {
        let a = MemberTree::new(&npks(3)).expect("non-empty").root();
        let mut reordered = npks(3);
        reordered.swap(0, 2);
        let b = MemberTree::new(&reordered).expect("non-empty").root();
        assert_ne!(a, b, "the member set is ordered; callers must fix an order");
    }
}
