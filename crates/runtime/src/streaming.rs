//! Streaming contracts (v0). Implement camera-centric loading with budgets.

#[derive(Debug, Clone, Copy)]
pub struct StreamingBudgets {
    pub max_tiles_loaded: usize,
    pub max_chunk_mesh_builds_per_frame: usize,
    pub max_io_requests_in_flight: usize,
}

impl Default for StreamingBudgets {
    fn default() -> Self {
        Self {
            max_tiles_loaded: 64,
            max_chunk_mesh_builds_per_frame: 2,
            max_io_requests_in_flight: 8,
        }
    }
}
