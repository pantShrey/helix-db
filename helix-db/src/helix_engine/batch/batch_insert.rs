use std::collections::HashMap;
use std::time::Instant;

use heed3::{RwTxn, PutFlags};
use bincode;
use tracing::{debug, error};

use crate::helix_engine::storage_core::{HelixGraphStorage, NodeId};
use crate::helix_engine::bm25::bm25::BM25;
use crate::helix_engine::types::GraphError;
use crate::utils::items::{Node, Edge};
use crate::utils::label_hash::hash_label;

use super::{BatchResult, BatchConfig};

impl HelixGraphStorage {
    /// Insert multiple nodes in a single transaction with optimized batch processing
    pub fn insert_nodes_batch(
        &self,
        nodes: Vec<Node>,
        config: &BatchConfig,
    ) -> Result<BatchResult, GraphError> {
        let start = Instant::now();
        let total_nodes = nodes.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        
        debug!("Starting batch insert of {} nodes", total_nodes);

        // Create a single transaction for the entire batch
        let mut txn = self.graph_env.write_txn()?;
        
        // Pre-allocate structures for batch operations
        let mut secondary_index_updates: HashMap<String, Vec<(Vec<u8>, NodeId)>> = HashMap::new();
        let mut bm25_documents = Vec::with_capacity(nodes.len());
        
        // Process all nodes first, collecting index updates
        for (idx, node) in nodes.into_iter().enumerate() {
            match self.process_single_node(&mut txn, node, &mut secondary_index_updates, &mut bm25_documents) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((idx, e));
                    if !config.auto_commit {
                        // If not auto-committing, fail fast on first error
                        return Err(GraphError::BatchInsertFailed(format!(
                            "Failed at index {}: {:?}", idx, errors.last().unwrap().1
                        )));
                    }
                }
            }
        }
        
        // Batch update secondary indices
        for (index_name, updates) in secondary_index_updates {
            if let Some(index_db) = self.secondary_indices.get(&index_name) {
                for (key, node_id) in updates {
                    if let Err(e) = index_db.put(&mut txn, &key, &node_id) {
                        error!("Failed to update secondary index {}: {:?}", index_name, e);
                        failed += 1;
                    }
                }
            }
        }
        
        // Batch update BM25 index
        if let Some(bm25) = &self.bm25 {
            for (node_id, text) in bm25_documents {
                if let Err(e) = bm25.insert_doc(&mut txn, node_id, &text) {
                    error!("Failed to update BM25 index for node {}: {:?}", node_id, e);
                    failed += 1;
                }
            }
        }
        
        // Commit the transaction
        txn.commit()?;
        
        let duration_ms = start.elapsed().as_millis();
        debug!("Batch insert completed in {}ms: {} successful, {} failed", 
               duration_ms, successful, failed);
        
        Ok(BatchResult {
            successful,
            failed,
            errors,
            duration_ms,
        })
    }
    
    /// Process a single node, collecting index updates for batch processing
    fn process_single_node(
        &self,
        txn: &mut RwTxn,
        node: Node,
        secondary_index_updates: &mut HashMap<String, Vec<(Vec<u8>, NodeId)>>,
        bm25_documents: &mut Vec<(NodeId, String)>,
    ) -> Result<(), GraphError> {
        // Serialize node data
        let bytes = bincode::serialize(&node)?;
        
        // Insert node data
        self.nodes_db.put_with_flags(
            txn,
            PutFlags::APPEND, // Use APPEND for better performance in batch inserts
            &node.id,
            &bytes,
        )?;
        
        // Collect secondary index updates
        if let Some(props) = &node.properties {
            for (key, value) in props.iter() {
                if self.secondary_indices.contains_key(key) {
                    let serialized = bincode::serialize(&(hash_label(&node.label, None), value))?;
                    secondary_index_updates
                        .entry(key.clone())
                        .or_insert_with(Vec::new)
                        .push((serialized, node.id));
                }
            }
        }
        
        // Collect BM25 document
        if self.bm25.is_some() {
            let mut text = String::new();
            if let Some(props) = &node.properties {
                // Manually flatten properties for BM25
                for (_, value) in props.iter() {
                    text.push_str(&format!("{} ", value));
                }
            }
            text.push_str(&node.label);
            bm25_documents.push((node.id, text));
        }
        
        Ok(())
    }

    /// Insert multiple edges in a single transaction with optimized batch processing
    pub fn insert_edges_batch(
        &self,
        edges: Vec<Edge>,
        config: &BatchConfig,
    ) -> Result<BatchResult, GraphError> {
        let start = Instant::now();
        let total_edges = edges.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut errors = Vec::new();
        
        debug!("Starting batch insert of {} edges", total_edges);

        // Create a single transaction for the entire batch
        let mut txn = self.graph_env.write_txn()?;
        
        // Pre-allocate buffers for edge indices
        let mut out_edge_updates = Vec::with_capacity(edges.len());
        let mut in_edge_updates = Vec::with_capacity(edges.len());
        
        // Process all edges first, collecting index updates
        for (idx, edge) in edges.into_iter().enumerate() {
            match self.process_single_edge(&mut txn, edge, &mut out_edge_updates, &mut in_edge_updates) {
                Ok(_) => successful += 1,
                Err(e) => {
                    failed += 1;
                    errors.push((idx, e));
                    if !config.auto_commit {
                        // If not auto-committing, fail fast on first error
                        return Err(GraphError::BatchInsertFailed(format!(
                            "Failed at index {}: {:?}", idx, errors.last().unwrap().1
                        )));
                    }
                }
            }
        }
        
        // Batch update out-edge indices
        for (key, value) in out_edge_updates {
            if let Err(e) = self.out_edges_db.put_with_flags(&mut txn, PutFlags::APPEND_DUP, &key, &value) {
                error!("Failed to update out-edge index: {:?}", e);
                failed += 1;
            }
        }
        
        // Batch update in-edge indices
        for (key, value) in in_edge_updates {
            if let Err(e) = self.in_edges_db.put_with_flags(&mut txn, PutFlags::APPEND_DUP, &key, &value) {
                error!("Failed to update in-edge index: {:?}", e);
                failed += 1;
            }
        }
        
        // Commit the transaction
        txn.commit()?;
        
        let duration_ms = start.elapsed().as_millis();
        debug!("Batch insert completed in {}ms: {} successful, {} failed", 
               duration_ms, successful, failed);
        
        Ok(BatchResult {
            successful,
            failed,
            errors,
            duration_ms,
        })
    }
    
    /// Process a single edge, collecting index updates for batch processing
    fn process_single_edge(
        &self,
        txn: &mut RwTxn,
        edge: Edge,
        out_edge_updates: &mut Vec<(Vec<u8>, Vec<u8>)>,
        in_edge_updates: &mut Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(), GraphError> {
        // Serialize edge data
        let bytes = bincode::serialize(&edge)?;
        
        // Insert edge data
        self.edges_db.put_with_flags(
            txn,
            PutFlags::APPEND,
            &edge.id,
            &bytes,
        )?;
        
        // Prepare out-edge index key and value
        let label_hash = hash_label(&edge.label, None);
        let mut out_key = Vec::with_capacity(24);
        out_key.extend_from_slice(&edge.from_node.to_be_bytes());
        out_key.extend_from_slice(&label_hash);
        
        let mut out_value = Vec::with_capacity(32);
        out_value.extend_from_slice(&edge.id.to_be_bytes());
        out_value.extend_from_slice(&edge.to_node.to_be_bytes());
        
        out_edge_updates.push((out_key, out_value));
        
        // Prepare in-edge index key and value
        let mut in_key = Vec::with_capacity(24);
        in_key.extend_from_slice(&edge.to_node.to_be_bytes());
        in_key.extend_from_slice(&label_hash);
        
        let mut in_value = Vec::with_capacity(32);
        in_value.extend_from_slice(&edge.id.to_be_bytes());
        in_value.extend_from_slice(&edge.from_node.to_be_bytes());
        
        in_edge_updates.push((in_key, in_value));
        
        Ok(())
    }
}