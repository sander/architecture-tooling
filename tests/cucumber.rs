mod knowledge_steps;

use cucumber::cucumber;
use tooling::knowledge::Dataset;

pub struct MyWorld {
    dataset: Option<Dataset>,
}

impl cucumber::World for MyWorld {}
impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        MyWorld {
            dataset: None::<Dataset>,
        }
    }
}

cucumber! {
    features: "./features",
    world: crate::MyWorld,
    steps: &[knowledge_steps::steps]
}
