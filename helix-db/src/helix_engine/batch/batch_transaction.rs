use std::time::Instant;

use heed3::{Env, RwTxn};
use tracing::{debug, warn};

use crate::helix_engine::types::GraphError;

/// A transaction wrapper that automatically commits when reaching a size threshold
pub struct BatchTransaction<'a> {
    env: &'a Env,
    txn: Option<RwTxn<'a>>,
    operations: usize,
    max_operations: usize,
    start_time: Instant,
    commit_count: usize,
}

impl<'a> BatchTransaction<'a> {
    /// Create a new batch transaction with configurable commit size
    pub fn new(env: &'a Env) -> Result<Self, GraphError> {
        let max_operations = std::env::var("HELIX_BATCH_COMMIT_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);
            
        let txn = env.write_txn()?;
        
        Ok(Self {
            env,
            txn: Some(txn),
            operations: 0,
            max_operations,
            start_time: Instant::now(),
            commit_count: 0,
        })
    }
    
    /// Get the current transaction, creating a new one if needed
    pub fn get_txn(&mut self) -> Result<&mut RwTxn<'a>, GraphError> {
        if self.txn.is_none() {
            self.txn = Some(self.env.write_txn()?);
        }
        
        Ok(self.txn.as_mut().unwrap())
    }
    
    /// Increment operation count and auto-commit if threshold reached
    pub fn increment_operations(&mut self, count: usize) -> Result<(), GraphError> {
        self.operations += count;
        
        if self.operations >= self.max_operations {
            self.commit()?;
        }
        
        Ok(())
    }
    
    /// Commit the current transaction and start a new one
    pub fn commit(&mut self) -> Result<(), GraphError> {
        if let Some(txn) = self.txn.take() {
            let commit_start = Instant::now();
            txn.commit()?;
            self.commit_count += 1;
            
            debug!(
                "Batch transaction committed {} operations in {:?} (commit #{})",
                self.operations,
                commit_start.elapsed(),
                self.commit_count
            );
            
            self.operations = 0;
            // Transaction will be created on next get_txn() call
        }
        
        Ok(())
    }
    
    /// Abort the current transaction
    pub fn abort(mut self) {
        if let Some(txn) = self.txn.take() {
            drop(txn); // Dropping without commit aborts the transaction
            warn!("Batch transaction aborted with {} pending operations", self.operations);
        }
    }
    
    /// Get statistics about the batch transaction
    pub fn stats(&self) -> BatchTransactionStats {
        BatchTransactionStats {
            operations: self.operations,
            commit_count: self.commit_count,
            elapsed: self.start_time.elapsed(),
        }
    }
}

impl<'a> Drop for BatchTransaction<'a> {
    fn drop(&mut self) {
        if self.txn.is_some() && self.operations > 0 {
            warn!(
                "BatchTransaction dropped with {} uncommitted operations. Auto-committing.",
                self.operations
            );
            let _ = self.commit();
        }
    }
}

/// Statistics for a batch transaction
#[derive(Debug)]
pub struct BatchTransactionStats {
    pub operations: usize,
    pub commit_count: usize,
    pub elapsed: std::time::Duration,
}

/// Extension trait for batch operations on storage
pub trait BatchStorageExt {
    /// Execute a batch operation with automatic transaction management
    fn execute_batch<F, R>(&self, batch_fn: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut BatchTransaction) -> Result<R, GraphError>;
}

// Example usage in a batch operation:
/*
impl BatchStorageExt for HelixGraphStorage {
    fn execute_batch<F, R>(&self, batch_fn: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut BatchTransaction) -> Result<R, GraphError>,
    {
        let mut batch_txn = BatchTransaction::new(&self.graph_env)?;
        let result = batch_fn(&mut batch_txn)?;
        batch_txn.commit()?;
        Ok(result)
    }
}
*/