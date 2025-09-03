use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    ops::Deref,
};

use heed3::{RoTxn, RwTxn, WithoutTls};

use crate::helix_engine::vector_core::vector::HVector;

pub struct VecTxn<'outer_scope, 'env> {
    pub txn: &'outer_scope mut RwTxn<'env>,
    pub cache: HashMap<(u128, usize), HashSet<HVector>>,
}

impl<'outer_scope, 'env> VecTxn<'outer_scope, 'env> {
    pub fn new(txn: &'outer_scope mut RwTxn<'env>) -> Self {
        Self {
            txn,
            cache: HashMap::with_capacity(4096),
        }
    }

    pub fn set_neighbors(&mut self, id: u128, level: usize, neighbors: &BinaryHeap<HVector>) {
        // get change sets in neighbors
        let neighbors = neighbors.iter().cloned().collect::<HashSet<_>>();

        if let Some(old_neighbors) = self.cache.get(&(id, level)) {
            let old_neighbors_to_delete = old_neighbors
                .difference(&neighbors)
                .cloned()
                .collect::<HashSet<_>>();

            for neighbor in old_neighbors_to_delete {
                if let Some(neighbor_set) = self
                    .cache
                    .get_mut(&(neighbor.get_id(), neighbor.get_level()))
                {
                    neighbor_set.remove(&neighbor);
                }
            }
        }

        self.cache.insert((id, level), neighbors);
    }

    pub fn get_neighbors(&self, id: u128, level: usize) -> Option<Vec<HVector>> {
        self.cache
            .get(&(id, level))
            .map(|x| x.iter().cloned().collect())
    }

    pub fn get_rtxn(&self) -> &RoTxn<'env, WithoutTls> {
        self.txn
    }

    pub fn get_wtxn(&mut self) -> &mut RwTxn<'env> {
        self.txn
    }
}

impl<'outer_scope, 'env> Deref for VecTxn<'outer_scope, 'env> {
    type Target = RoTxn<'env, WithoutTls>;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}
