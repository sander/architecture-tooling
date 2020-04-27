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
            let local = url::Url::parse("http://localhost:3030/").unwrap();
            let knowledge = FusekiKnowledgeService {
                client: &client,
                base: local,
            };
            let dataset = block_on(knowledge.create_temporary_dataset());
            world.dataset = Some(dataset);
        })
        .then("I have a dataset", |world, _step| {
            assert!(world.dataset.is_some())
        });

    builder.build()
}
