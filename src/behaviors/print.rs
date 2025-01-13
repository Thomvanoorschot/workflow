use super::NodeBehavior;

#[derive(Clone)]
pub struct PrintBehavior;

impl NodeBehavior for PrintBehavior {
    fn on_active(&self) -> bool {
        println!("DONE");
        true
    }

    fn on_completed(&self) -> bool {
        true
    }
}
