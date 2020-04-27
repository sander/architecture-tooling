mod knowledge_steps;

use cucumber::cucumber;
use tooling::knowledge::Dataset;
use tooling::knowledge::FusekiKnowledgeService;
use tooling::knowledge::KnowledgeService;

pub struct MyWorld {
    foo: String,
    dataset: Option<Dataset>,
}

impl cucumber::World for MyWorld {}
impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        // let client = reqwest::Client::new();
        // let local = url::Url::parse("http://localhost:3030/").unwrap();
        // let knowledge = FusekiKnowledgeService {
        //     client: &client,
        //     base: local,
        // };

        MyWorld {
            foo: "a default string".to_string(),
            dataset: None::<Dataset>,
            // knowledge,
        }
    }
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for knowledge_steps and the main cucumber call
    steps: &[
        knowledge_steps::steps // the `knowledge_steps!` macro creates a `knowledge_steps` function in a module
    ]
}
