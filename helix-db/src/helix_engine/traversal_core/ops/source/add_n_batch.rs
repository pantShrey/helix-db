use crate::{
    helix_engine::{
        batch::{BatchConfig, BatchResult},
        traversal_core::{traversal_iter::RwTraversalIterator, traversal_value::TraversalValue},
        types::GraphError,
    },
    protocol::value::Value,
    utils::{id::v6_uuid, items::Node},
};

pub struct AddNBatchIterator {
    inner: std::iter::Once<Result<TraversalValue, GraphError>>,
}

impl Iterator for AddNBatchIterator {
    type Item = Result<TraversalValue, GraphError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub trait AddNBatchAdapter<'a, 'b>: Iterator<Item = Result<TraversalValue, GraphError>> {
    /// Add multiple nodes in a single batch operation
    fn add_n_batch(
        self,
        nodes_data: Vec<(String, Option<Vec<(String, Value)>>)>, // (label, properties)
        _secondary_indices: Option<&'a [&str]>,
    ) -> RwTraversalIterator<'a, 'b, std::iter::Once<Result<TraversalValue, GraphError>>>;
}

impl<'a, 'b, I: Iterator<Item = Result<TraversalValue, GraphError>>> AddNBatchAdapter<'a, 'b>
    for RwTraversalIterator<'a, 'b, I>
{
    fn add_n_batch(
        self,
        nodes_data: Vec<(String, Option<Vec<(String, Value)>>)>,
        _secondary_indices: Option<&'a [&str]>,
    ) -> RwTraversalIterator<'a, 'b, std::iter::Once<Result<TraversalValue, GraphError>>> {
        
        // Create batch configuration from environment
        let batch_config = BatchConfig::from_env();
        
        // Prepare nodes for batch insert
        let mut nodes = Vec::with_capacity(nodes_data.len());
        let mut node_ids = Vec::with_capacity(nodes_data.len());
        
        for (label, properties) in nodes_data {
            let node = Node {
                id: v6_uuid(),
                label,
                version: 1,
                properties: properties.map(|props| props.into_iter().collect()),
            };
            node_ids.push(node.id);
            nodes.push(node);
        }
        
        // Perform batch insert
        let batch_result = self
            .storage
            .insert_nodes_batch(nodes, &batch_config)
            .unwrap_or_else(|e| BatchResult {
                successful: 0,
                failed: node_ids.len(),
                errors: vec![(0, e)],
                duration_ms: 0,
            });
        
        // Convert batch result to traversal value
        let result = if batch_result.failed == 0 {
            // All successful - return count
            Ok(TraversalValue::Count(crate::utils::count::Count::new(
                batch_result.successful
            )))
        } else if batch_result.successful == 0 {
            // All failed
            Err(GraphError::BatchInsertFailed(format!(
                "All {} nodes failed to insert", batch_result.failed
            )))
        } else {
            // Partial success - still return count but log warning
            tracing::warn!(
                "Batch insert partially succeeded: {} successful, {} failed",
                batch_result.successful,
                batch_result.failed
            );
            Ok(TraversalValue::Count(crate::utils::count::Count::new(
                batch_result.successful
            )))
        };
        
        RwTraversalIterator::new(self.storage, self.txn, std::iter::once(result))
    }
}