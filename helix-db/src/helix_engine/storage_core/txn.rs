use std::sync::Arc;

use crate::helix_engine::{storage_core::HelixGraphStorage, types::GraphError};

pub trait HelixTxn {
    fn new_rtxn<'a>(env: &'a Arc<HelixGraphStorage>) -> Result<Self, GraphError>;
    fn new_wtxn<'a>(env: &'a Arc<HelixGraphStorage>) -> Result<Self, GraphError>;
}