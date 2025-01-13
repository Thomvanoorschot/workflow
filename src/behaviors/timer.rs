use super::NodeBehavior;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct TimerBehavior {
    request_id: Arc<Mutex<Option<String>>>,
}

impl TimerBehavior {
    pub fn new() -> Self {
        Self {
            request_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_request_id(&self) -> Option<String> {
        self.request_id.lock().clone()
    }
}

impl NodeBehavior for TimerBehavior {
    fn on_active(&self) -> bool {
        let id = format!("timer_{}", rand::random::<u32>());
        println!("Generated timer ID: {}", id);
        *self.request_id.lock() = Some("1".to_string());
        true
    }

    fn on_completed(&self) -> bool {
        true
    }
}
