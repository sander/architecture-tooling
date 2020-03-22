mod architecture;
mod knowledge;

use crate::knowledge::KnowledgeService;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let local = url::Url::parse("http://localhost:3030/")?;
    let service = knowledge::FusekiKnowledgeService::new(&client, local);
    let dataset = service.create_temporary_dataset().await;
    println!("dataset: {:?}", dataset);

    let togaf_url = "https://raw.githubusercontent.com/sander/togaf-content-metamodel-ontology/master/OntologyTOGAFContentMetamodelV1.xml";
    let togaf_body = client.get(togaf_url).send().await?.bytes().await?;
    let togaf_file = knowledge::DataFile::RdfXml(togaf_body.to_vec());

    let archi_contents = fs::read("architecture.ttl").unwrap();
    let archi_file = knowledge::DataFile::Turtle(archi_contents);

    service.import(&dataset, togaf_file).await;
    service.import(&dataset, archi_file).await;

    Ok(())
}
