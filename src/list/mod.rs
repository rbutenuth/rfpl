use std::sync::Arc;

use crate::Value;

mod construct;
mod destruct;

#[derive(Debug)]
pub struct FplList {
    buckets: Arc<[Bucket]>,
}


#[derive(Debug)]
struct Bucket {
    values: Arc<[Value]>,
}

impl Clone for FplList {
    fn clone(&self) -> Self {
        Self { buckets: Arc::clone(&self.buckets) }
    }
}