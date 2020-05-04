use std::fs;
use std::io::Write;
use std::process;
use tooling::architecture::{visualization, ArchitectureService};
use tooling::knowledge::{DataFile, KnowledgeService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let local = url::Url::parse("http://localhost:3030/")?;
    let knowledge = tooling::knowledge::FusekiKnowledgeService {
        client: &client,
        base: local,
    };
    let name = "architecture";
    let dataset = knowledge.create_dataset(name.to_string()).await;
    println!("dataset: {:?}", dataset);

    let togaf_contents = fs::read("OntologyTOGAFContentMetamodelV1.xml").unwrap();
    let togaf_file = DataFile::RdfXml(togaf_contents);

    let archi_contents = fs::read("architecture.ttl").unwrap();
    let archi_file = DataFile::Turtle(archi_contents);

    knowledge
        .import(&dataset, "togaf".to_string(), togaf_file)
        .await;
    knowledge
        .import(&dataset, "architecture.ttl".to_string(), archi_file)
        .await;

    let architecture = tooling::architecture::DataBackedArchitectureService {
        dataset: &dataset,
        knowledge: &knowledge,
    };
    let components = architecture.components().await;
    println!("result: {:?}", components);

    let relations = architecture.relations().await;
    println!("relations: {:?}", relations);

    let visualization = visualization(components, relations);

    let child = process::Command::new("dot")
        .arg("-Tsvg")
        .arg("-odoc/example.svg")
        .stdin(process::Stdio::piped())
        .spawn()
        .expect("failed to execute dot");

    let mut outstdin = child.stdin.expect("failed to obtain stdin");
    outstdin
        .write_all(visualization.graphviz_content.as_bytes())
        .expect("failed to write");

    println!("graph: {:?}", visualization);

    let ids = architecture.component_ids().await;
    println!("ids: {:?}", ids);

    let descriptions = ids.iter().map(|id| architecture.describe(id));
    let foo = futures::future::join_all(descriptions).await;
    println!("descriptions: {:?}", foo);

    Ok(())
}
