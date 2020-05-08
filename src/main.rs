use graph_store::{doc, DataFile, Graph, GraphStore, Resource};
use std::fs;
use std::io::Write;
use std::process;
use tooling::architecture::{visualization, ArchitectureService};
use tooling::knowledge::KnowledgeService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let dataset = graph_store::http::Dataset::get_or_create(
        &client,
        url::Url::parse("http://localhost:3030").unwrap(),
        "architecture",
    )
    .await;

    let client_for_deprecated_service = reqwest::Client::new();
    let knowledge = tooling::knowledge::FusekiKnowledgeService::new(
        &client_for_deprecated_service,
        "http://localhost:3030/",
    );
    let deprecated_dataset = knowledge.create_dataset("architecture".to_string()).await;

    dataset
        .import(
            Graph::Named(Resource::from("togaf")),
            DataFile::unsafe_from_turtle(
                &fs::read_to_string("OntologyTOGAFContentMetamodelV2.ttl").unwrap(),
            ),
        )
        .await;
    dataset
        .import(
            Graph::Named(Resource::from("architecture.ttl")),
            DataFile::unsafe_from_turtle(&fs::read_to_string("architecture.ttl").unwrap()),
        )
        .await;

    let architecture = tooling::architecture::DataBackedArchitectureService {
        dataset: &deprecated_dataset,
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

    doc::export_to_html(&dataset).await;

    Ok(())
}
