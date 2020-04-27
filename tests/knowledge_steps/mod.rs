use cucumber::steps;
use tooling::knowledge::FusekiKnowledgeService;
use tooling::knowledge::KnowledgeService;

// Any type that implements cucumber::World + Default can be the world
steps!(crate::MyWorld => {

    given "I have no dataset" |world, step| {
        world.dataset = None;
    };

    when "I create a temporary dataset" |world, step| {
        let client = reqwest::Client::new();
        let local = url::Url::parse("http://localhost:3030/").unwrap();
        let knowledge = FusekiKnowledgeService {
            client: &client,
            base: local,
        };
        let dataset = knowledge.create_temporary_dataset();
        // world.dataset = Some(dataset);
    };
});
