use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    ops::Deref,
    rc::Rc,
};

use heed3::{
    Database, Env, RoTxn, RwTxn, WithoutTls,
    types::{Bytes, Unit},
};
use itertools::Itertools;

use crate::{
    helix_engine::{
        types::VectorError,
        vector_core::{vector::HVector, vector_core::VectorCore},
    },
    protocol::value::Value,
};

/// VecTxn provides a transaction-scoped cache for vector operations.
///
/// ## In-Memory Preloading
///
/// VecTxn supports preloading all existing vectors and edges into memory
/// for faster access. This is useful when you need to perform many vector
/// operations and want to avoid repeated LMDB reads.
///
/// ### Example Usage:
///
/// ```rust
/// // Standard usage (on-demand loading from LMDB)
/// let mut vec_txn = VecTxn::new(env.write_txn()?);
///
/// // With preloading (loads all vectors/edges into memory)
/// let mut vec_txn = VecTxn::new_with_preload(
///     env.write_txn()?,
///     &vector_core
/// )?;
///
/// // Alternatively, preload after creation
/// let mut vec_txn = VecTxn::new(env.write_txn()?);
/// vec_txn.preload_all(&vector_core)?;
///
/// // Use the transaction for vector operations
/// let results = vector_core.search_with_vec_txn(
///     &mut vec_txn,
///     query_vector,
///     10,  // k
///     "label",
///     None,  // filter
///     false  // should_trickle
/// )?;
///
/// // Commit the transaction
/// vec_txn.commit(&vector_core.edges_db)?;
/// ```
pub struct VecTxn {
    pub cache: HashMap<(u128, usize), HashSet<Rc<HVector>>>,
    // Preloaded vectors cache - stores all vectors when preload is enabled
    pub vectors_cache: HashMap<(u128, usize), Rc<HVector>>,
    // Whether this transaction has preloaded all data
    pub is_preloaded: bool,
}

impl VecTxn {
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(256),
            vectors_cache: HashMap::new(),
            is_preloaded: false,
        }
    }

    /// Create a new VecTxn and preload all vectors and edges into memory
    pub fn new_with_preload(txn: &RoTxn, vector_core: &VectorCore) -> Result<Self, VectorError> {
        let vectors_len = vector_core.vectors_db.len(&txn)? as usize;
        let edges_len = vector_core.edges_db.len(&txn)? as usize;

        let mut vec_txn = Self {
            cache: HashMap::with_capacity(edges_len),
            vectors_cache: HashMap::with_capacity(vectors_len),
            is_preloaded: false,
        };

        // Preload all data
        vec_txn.preload_all(txn, vector_core)?;
        Ok(vec_txn)
    }

    /// Preload all vectors and edges from LMDB into memory
    pub fn preload_all(
        &mut self,
        txn: &RoTxn,
        vector_core: &VectorCore,
    ) -> Result<(), VectorError> {
        println!("Preloading vectors and edges into VecTxn memory...");

        // Clear existing caches
        self.cache.clear();
        self.vectors_cache.clear();

        // Load all vectors
        let mut loaded_vectors = 0;
        let iter = vector_core
            .vectors_db
            .prefix_iter(&txn, vector_core.get_vector_prefix())?;

        for result in iter {
            let (key, value) = result?;

            // Skip non-vector entries (like entry_point)
            if !key.starts_with(vector_core.get_vector_prefix()) {
                continue;
            }

            // Parse key to get id and level
            let prefix_len = vector_core.get_vector_prefix().len();
            if key.len() >= prefix_len + 24 {
                // prefix + id(16) + level(8)
                let id_bytes = &key[prefix_len..prefix_len + 16];
                let level_bytes = &key[prefix_len + 16..prefix_len + 24];

                let id = u128::from_be_bytes(id_bytes.try_into().unwrap());
                let level = usize::from_be_bytes(level_bytes.try_into().unwrap());

                // Deserialize vector
                let mut vector = HVector::from_bytes(id, level, value)?;

                // Load properties if they exist
                if let Ok(Some(data)) = vector_core.vector_data_db.get(&txn, &id.to_be_bytes()) {
                    let properties: HashMap<String, Value> = bincode::deserialize(&data)?;
                    vector.properties = Some(properties);
                }

                // Store in vectors cache
                self.vectors_cache.insert((id, level), Rc::new(vector));
                loaded_vectors += 1;
            }
        }

        // Load all edges into neighbor cache
        let mut loaded_edges = 0;
        let iter = vector_core.edges_db.iter(&txn)?;

        for result in iter {
            let (key, _) = result?;

            // Parse edge key: source_id(16) + level(8) + sink_id(16)
            if key.len() == 40 {
                // 16 + 8 + 16 bytes
                let source_id = u128::from_be_bytes(key[0..16].try_into().unwrap());
                let level = usize::from_be_bytes(key[16..24].try_into().unwrap());
                let sink_id = u128::from_be_bytes(key[24..40].try_into().unwrap());

                // Get the neighbor vector from vectors_cache
                if let Some(neighbor_vec) = self.vectors_cache.get(&(sink_id, level)) {
                    self.cache
                        .entry((source_id, level))
                        .or_insert_with(HashSet::new)
                        .insert(Rc::clone(neighbor_vec));
                    loaded_edges += 1;
                }
            }
        }

        println!(
            "Preloaded {} vectors and {} edges into VecTxn memory",
            loaded_vectors, loaded_edges
        );

        self.is_preloaded = true;
        Ok(())
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
            .remove(&(curr_vec.get_id(), level))
            .unwrap_or_default();

        for old_neighbor in &old_neighbors {
            if neighbors.contains(old_neighbor) {
                continue;
            }
            if let Some(neighbor_set) = self.cache.get_mut(&(old_neighbor.get_id(), level)) {
                neighbor_set.remove(&curr_vec);
            }
        }

        for neighbor in &neighbors {
            if neighbor.get_id() == curr_vec.get_id() || old_neighbors.contains(neighbor) {
                continue;
            }
            self.cache
                .entry((neighbor.get_id(), level))
                .or_insert_with(HashSet::new)
                .insert(Rc::clone(&curr_vec));
        }

        self.cache.insert((curr_vec.get_id(), level), neighbors);
    }

    pub fn get_neighbors(&self, id: u128, level: usize) -> Option<Vec<Rc<HVector>>> {
        // First check the neighbors cache (which includes preloaded edges and modifications)
        self.cache
            .get(&(id, level))
            .map(|x| x.iter().map(Rc::clone).collect())
    }

    /// Get a vector from cache if preloaded
    pub fn vec_txn_get_vector(&self, id: u128, level: usize) -> Option<Rc<HVector>> {
        self.vectors_cache.get(&(id, level)).map(Rc::clone)
    }

    pub fn vec_txn_put_vector(&mut self, vector: &HVector, level: usize) {
        let mut vector_at_level = vector.clone();
        vector_at_level.level = level;
        let vector_at_level = Rc::new(vector_at_level);
        self.vectors_cache
            .insert((vector_at_level.get_id(), level), vector_at_level.clone());

        if level > 0 {
            let mut vector_at_0 = vector.clone();
            vector_at_0.level = 0;
            self.vectors_cache
                .insert((vector_at_0.get_id(), 0), Rc::new(vector_at_0));
        }
    }

    pub fn insert_neighbors(&mut self, id: u128, level: usize, neighbors: &Vec<Rc<HVector>>) {
        let neighbors = neighbors.iter().map(Rc::clone).collect::<HashSet<_>>();
        self.cache.entry((id, level)).or_default().extend(neighbors);
    }

    pub fn commit(self, env: &Env, vector_core: &VectorCore) -> Result<(), VectorError> {
        let mut txn = env.write_txn()?;
        let vec_len = vector_core.vectors_db.len(&txn)? as usize;
        let edge_len = vector_core.edges_db.len(&txn)? as usize;
        println!("Existing LMDB: {} vectors, {} edges", vec_len, edge_len);

        // get entry point
        let entry_point = vector_core.get_entry_point(&txn)?;

        println!("Wiping old data from LMDB...");
        vector_core.vectors_db.clear(&mut txn)?;
        vector_core.edges_db.clear(&mut txn)?;

        // put entry point
        vector_core.set_entry_point(&mut txn, &entry_point)?;

        txn.commit()?;

        println!("Putting vectors into LMDB...");
        println!("Vectors cache size: {}", self.vectors_cache.len());
        println!(
            "Cache size: {}",
            self.cache.values().map(|x| x.len()).sum::<usize>()
        );

        let vector_chunk_size = 250_000;
        for chunk in &self
            .vectors_cache
            .iter()
            .sorted_by(|(_, a), (_, b)| a.get_id().cmp(&b.get_id()))
            .chunks(vector_chunk_size)
        {
            let mut txn = env.write_txn()?;
            for ((id, level), vector) in chunk {
                vector_core.vectors_db.put(
                    &mut txn,
                    &VectorCore::vector_key(*id, *level),
                    &vector.to_bytes(),
                )?;
                if let Some(properties) = &vector.properties {
                    vector_core.vector_data_db.put(
                        &mut txn,
                        &id.to_be_bytes(),
                        &bincode::serialize(&properties)?,
                    )?;
                }
            }
            txn.commit()?;
        }

        let chunk_size = 1_000_000;
        for chunk in &self.cache.iter().chunks(chunk_size) {
            let mut txn = env.write_txn()?;
            let mut vec = HashSet::with_capacity(chunk_size);
            for ((id, level), neighbours) in chunk {
                for neighbor in neighbours {
                    if neighbor.get_id() == *id {
                        continue;
                    }
                    let out_key = VectorCore::out_edges_key(*id, *level, Some(neighbor.get_id()));
                    let in_key = VectorCore::out_edges_key(neighbor.get_id(), *level, Some(*id));
                    vec.insert(out_key);
                    vec.insert(in_key);
                }
            }
            for key in vec.into_iter().sorted() {
                vector_core.edges_db.put(&mut txn, &key, &())?;
            }
            txn.commit()?;
        }
        Ok(())
    }
}
