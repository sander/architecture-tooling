mod architecture;
mod knowledge;

async fn create_dataset_overwriting_any_existing_one<S>(
    service: S,
    name: &str,
) -> Result<knowledge::Dataset, String>
where
    S: knowledge::Service,
{
    match service.create(name).await {
        Ok(d) => Ok(d),
        Err(e) => match e {
            knowledge::DatasetCreationError::NameAlreadyTaken(d) => match service.delete(d).await {
                Ok(_) => match service.create(name).await {
                    Err(knowledge::DatasetCreationError::IO(s)) => Err(s),
                    Err(knowledge::DatasetCreationError::NameAlreadyTaken(_)) => {
                        Err("race condition".to_owned())
                    }
                    Ok(d) => Ok(d),
                },
                Err(f) => Err(format!("{:?}", f)),
            },
            knowledge::DatasetCreationError::IO(s) => Err(s),
        },
    }
}

async fn program<S: knowledge::Service>(service: S) {
    let d = create_dataset_overwriting_any_existing_one(service, "myname2")
        .await
        .unwrap();
    println!("service: {:?}", d);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let local = url::Url::parse("http://localhost:3030/")?;
    let service = knowledge::live::FusekiKnowledgeService::new(client, local);
    program(service).await;
    Ok(())
}
