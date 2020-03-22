mod architecture;
mod knowledge;

use architecture::ArchitectureService;
use knowledge::KnowledgeService;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let local = url::Url::parse("http://localhost:3030/")?;
    let knowledge = knowledge::FusekiKnowledgeService::new(&client, local);
    let dataset = knowledge.create_temporary_dataset().await;
    println!("dataset: {:?}", dataset);

    // let togaf_url = "https://raw.githubusercontent.com/sander/togaf-content-metamodel-ontology/master/OntologyTOGAFContentMetamodelV1.xml";
    // let togaf_body = client.get(togaf_url).send().await?.bytes().await?;
    // let togaf_file = knowledge::DataFile::RdfXml(togaf_body.to_vec());
    let togaf_contents = fs::read("OntologyTOGAFContentMetamodelV1.xml").unwrap();
    let togaf_file = knowledge::DataFile::RdfXml(togaf_contents);

    let archi_contents = fs::read("architecture.ttl").unwrap();
    let archi_file = knowledge::DataFile::Turtle(archi_contents);

    knowledge.import(&dataset, togaf_file).await;
    knowledge.import(&dataset, archi_file).await;

    let architecture = architecture::DataBackedArchitectureService::new(&dataset, &knowledge);
    let result = architecture.components().await;
    println!("result: {:?}", result);

    knowledge.delete(&dataset).await;

    Ok(())
}
