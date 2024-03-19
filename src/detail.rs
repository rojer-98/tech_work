use std::sync::Arc;

use tokio::sync::RwLock;

use crate::utils::generate_unique_id;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DetailState {
    #[default]
    Start,
    Prepared,
    Procesed,
    Broken,
}

#[derive(Debug, Default, Clone)]
pub struct DetailInfo {
    pub mech_id: usize,
    pub detail: Arc<RwLock<Detail>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Detail {
    pub id: usize,
    pub state: DetailState,
}

impl Detail {
    pub fn new() -> Self {
        Self {
            id: generate_unique_id(),
            ..Default::default()
        }
    }
}
