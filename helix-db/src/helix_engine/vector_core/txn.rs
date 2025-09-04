use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    ops::Deref,
    rc::Rc,
};

use heed3::{
    Database, PutFlags, RoTxn, RwTxn, WithoutTls,
    types::{Bytes, Unit},
};

use crate::helix_engine::{
    types::VectorError,
    vector_core::{vector::HVector, vector_core::VectorCore},
};

pub struct VecTxn<'env> {
    pub txn: RwTxn<'env>,
    pub cache: HashMap<(u128, usize), HashSet<Rc<HVector>>>,
    pub cache_distances: HashMap<(u128, usize), f64>,
}

impl<'env> VecTxn<'env> {
    pub fn new(txn: RwTxn<'env>) -> Self {
        Self {
            txn,
            cache: HashMap::with_capacity(256),
            cache_distances: HashMap::with_capacity(256),
        }
    }

    pub fn set_neighbors(
        &mut self,
        curr_vec: Rc<HVector>,
        level: usize,
        neighbors: &BinaryHeap<Rc<HVector>>,
    ) {
        // get change sets in neighbors
        let neighbors = neighbors.iter().map(Rc::clone).collect::<HashSet<_>>();

        let old_neighbors = self
            .cache
            .get(&(curr_vec.get_id(), level))
            .cloned()
            .unwrap_or_default();
        let old_neighbors_to_delete = old_neighbors
            .difference(&neighbors)
            .map(Rc::clone)
            .collect::<HashSet<_>>();

        let neighbors_to_add = neighbors
            .difference(&old_neighbors)
            .map(Rc::clone)
            .collect::<HashSet<_>>();

        for neighbor in old_neighbors_to_delete {
            if let Some(neighbor_set) = self.cache.get_mut(&(neighbor.get_id(), level)) {
                neighbor_set.remove(&curr_vec);
            }
        }

        for neighbor in neighbors_to_add {
            self.cache
                .entry((neighbor.get_id(), level))
                .or_insert_with(HashSet::new)
                .insert(Rc::clone(&curr_vec));
        }

        self.cache.insert((curr_vec.get_id(), level), neighbors);
    }

    pub fn set_distance(&mut self, id: u128, level: usize, distance: f64) {
        self.cache_distances.insert((id, level), distance);
    }
    pub fn get_distance(&self, id: u128, level: usize) -> f64 {
        *self.cache_distances.get(&(id, level)).unwrap_or(&2.0)
    }

    pub fn get_neighbors(&self, id: u128, level: usize) -> Option<Vec<Rc<HVector>>> {
        self.cache
            .get(&(id, level))
            .map(|x| x.iter().map(Rc::clone).collect())
    }

    pub fn insert_neighbors(&mut self, id: u128, level: usize, neighbors: &Vec<Rc<HVector>>) {
        let neighbors = neighbors.iter().map(Rc::clone).collect::<HashSet<_>>();
        self.cache
            .entry((id, level))
            .and_modify(|x| x.extend(neighbors.clone()))
            .or_insert(neighbors);
    }

    pub fn get_rtxn(&self) -> &RoTxn<'env, WithoutTls> {
        &self.txn
    }

    pub fn get_wtxn(&mut self) -> &mut RwTxn<'env> {
        &mut self.txn
    }

    pub fn commit(mut self, db: &Database<Bytes, Unit>) -> Result<(), VectorError> {
        let txn = &mut self.txn;
        let mut vec = Vec::with_capacity(self.cache.len() * 128);
        let mut vecs = 0;
        for (id, level) in self.cache.keys() {
            if let Some(neighbors) = self.cache.get(&(*id, *level)) {
                for neighbor in neighbors {
                    let out_key = VectorCore::out_edges_key(*id, *level, Some(neighbor.get_id()));
                    vec.push(out_key);
                }
                vecs += 1;
            }
        }
        // vec.sort();
        println!("inserting: {:?}", vec.len());
        println!("vecs: {:?}", vecs);
        for key in vec {
            // db.put_with_flags(txn, PutFlags::APPEND, &key, &())?;
            db.put(txn, &key, &())?;
        }

        self.txn.commit().map_err(VectorError::from)
    }
}

impl<'env> Deref for VecTxn<'env> {
    type Target = RoTxn<'env, WithoutTls>;

    fn deref(&self) -> &Self::Target {
        &self.txn
    }
}
