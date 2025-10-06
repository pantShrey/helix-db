use crate::{
    helix_engine::{
        batch::{BatchConfig, BatchResult},
        storage_core::NodeId,
        traversal_core::{traversal_iter::RwTraversalIterator, traversal_value::TraversalValue},
        types::GraphError,
    },
    protocol::value::Value,
    utils::{id::v6_uuid, items::Edge},
};

pub struct AddEBatchIterator {
    inner: std::iter::Once<Result<TraversalValue, GraphError>>,
}

impl Iterator for AddEBatchIterator {
    type Item = Result<TraversalValue, GraphError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub trait AddEBatchAdapter<'a, 'b>: Iterator<Item = Result<TraversalValue, GraphError>> {
    /// Add multiple edges in a single batch operation
    fn add_e_batch(
        self,
        edges_data: Vec<(NodeId, NodeId, String, Option<Vec<(String, Value)>>)>, // (from, to, label, properties)
    ) -> RwTraversalIterator<'a, 'b, std::iter::Once<Result<TraversalValue, GraphError>>>;
}

impl<'a, 'b, I: Iterator<Item = Result<TraversalValue, GraphError>>> AddEBatchAdapter<'a, 'b>
    for RwTraversalIterator<'a, 'b, I>
{
    fn add_e_batch(
        self,
        edges_data: Vec<(NodeId, NodeId, String, Option<Vec<(String, Value)>>)>,
    ) -> RwTraversalIterator<'a, 'b, std::iter::Once<Result<TraversalValue, GraphError>>> {
        // Create batch configuration from environment
        let batch_config = BatchConfig::from_env();

        // Validate nodes exist before creating edges
        let rtxn = self.storage.graph_env.read_txn().unwrap();
        let mut edges = Vec::with_capacity(edges_data.len());
        let mut edge_ids = Vec::with_capacity(edges_data.len());
        let mut validation_errors = Vec::new();

        for (idx, (from_id, to_id, label, properties)) in edges_data.into_iter().enumerate() {
            // Check if from node exists
            let from_exists = self
                .storage
                .nodes_db
                .get(&rtxn, &from_id)
                .unwrap_or(None)
                .is_some();
            let to_exists = self
                .storage
                .nodes_db
                .get(&rtxn, &to_id)
                .unwrap_or(None)
                .is_some();

            if !from_exists {
                validation_errors.push((idx, GraphError::NodeNotFound));
                continue;
            }
            if !to_exists {
                validation_errors.push((idx, GraphError::NodeNotFound));
                continue;
            }

            let edge = Edge {
                id: v6_uuid(),
                from_node: from_id,
                to_node: to_id,
                label,
                version: 1,
                properties: properties.map(|props| props.into_iter().collect()),
            };
            edge_ids.push(edge.id);
            edges.push(edge);
        }

        drop(rtxn); // Release read transaction before write

        // Perform batch insert only if we have valid edges
        let result = if edges.is_empty() {
            Err(GraphError::BatchInsertFailed(format!(
                "No valid edges to insert. {} validation errors occurred",
                validation_errors.len()
            )))
        } else {
            let batch_result = self
                .storage
                .insert_edges_batch(edges, &batch_config)
                .unwrap_or_else(|e| BatchResult {
                    successful: 0,
                    failed: edge_ids.len(),
                    errors: vec![(0, e)],
                    duration_ms: 0,
                });

            if batch_result.failed == 0 && validation_errors.is_empty() {
                // All successful
                Ok(TraversalValue::Count(crate::utils::count::Count::new(
                    batch_result.successful
                )))
            } else if batch_result.successful == 0 {
                // All failed
                Err(GraphError::BatchInsertFailed(format!(
                    "All {} edges failed to insert",
                    batch_result.failed
                )))
            } else {
                // Partial success
                let total_failed = batch_result.failed + validation_errors.len();
                tracing::warn!(
                    "Batch insert partially succeeded: {} successful, {} failed (including {} validation errors)",
                    batch_result.successful,
                    total_failed,
                    validation_errors.len()
                );
                Ok(TraversalValue::Count(crate::utils::count::Count::new(
                    batch_result.successful
                )))
            }
        };

        RwTraversalIterator::new(self.storage, self.txn, std::iter::once(result))
    }
}
