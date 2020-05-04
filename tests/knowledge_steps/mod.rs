use cucumber::{Steps, StepsBuilder};
use tokio_test::block_on;
use tooling::knowledge::FusekiKnowledgeService;
use tooling::knowledge::KnowledgeService;

pub fn steps() -> Steps<crate::MyWorld> {
    let mut builder: StepsBuilder<crate::MyWorld> = StepsBuilder::new();

    builder
        .given("I have no dataset", |world, _step| {
            world.dataset = None;
        })
        .when("I create a temporary dataset", |world, _step| {
            let client = reqwest::Client::new();
            let knowledge = FusekiKnowledgeService::new(&client, "http://localhost:3030/");
            let name = "test".to_string();
            let dataset = block_on(knowledge.create_dataset(name));
            world.dataset = Some(dataset);
        })
        .then("I have a dataset", |world, _step| {
            assert!(world.dataset.is_some())
        });

    builder.build()
}
