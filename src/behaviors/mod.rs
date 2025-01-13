mod print;
mod timer;

pub use print::PrintBehavior;
pub use timer::TimerBehavior;

pub trait NodeBehavior: Send + Sync {
    fn on_active(&self) -> bool;
    fn on_completed(&self) -> bool;
} 