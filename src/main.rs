mod architecture;
mod knowledge;

use crate::architecture::ComponentKind;
use architecture::ArchitectureService;
use knowledge::KnowledgeService;
use std::fs;
use std::io::Write;
use std::process;

fn graph<'a>(
    components: Vec<architecture::Component>,
    relations: Vec<architecture::Relation>,
) -> String {
    let mut s = "digraph components_relations {\n".to_string();

    for c in components.iter() {
        s.push('"');
        s.push_str(c.label.replace('"', "\\\"").as_str());
        s.push_str("\" [shape=plain,style=filled,label=<<B>");
        s.push_str(c.label.replace('"', "\\\"").as_str());
        s.push_str("</B><BR/>[");
        s.push_str(match c.kind {
            ComponentKind::BusinessService => "Business service",
            ComponentKind::Function => "Function",
            ComponentKind::InformationSystemService => "Information system service",
            ComponentKind::Process => "Process",
        });
        s.push_str("]");
        match &c.description {
            None => {}
            Some(d) => {
                s.push_str("<BR/><BR/>");
                s.push_str(d.replace('"', "\\\"").as_str());
            }
        }
        s.push_str(">]\n");
    }

    for r in relations.iter() {
        s.push('"');
        s.push_str(r.from.replace('"', "\\\"").as_str());
        s.push_str("\" -> \"");
        s.push_str(r.to.replace('"', "\\\"").as_str());
        s.push_str("\" [label=\"");
        s.push_str(r.label.replace('"', "\\\"").as_str());
        s.push_str("\"]\n");
    }

    s.push_str("}");

    s
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let local = url::Url::parse("http://localhost:3030/")?;
    let knowledge = knowledge::FusekiKnowledgeService {
        client: &client,
        base: local,
    };
    let dataset = knowledge.create_temporary_dataset().await;
    println!("dataset: {:?}", dataset);

    let togaf_contents = fs::read("OntologyTOGAFContentMetamodelV1.xml").unwrap();
    let togaf_file = knowledge::DataFile::RdfXml(togaf_contents);

    let archi_contents = fs::read("architecture.ttl").unwrap();
    let archi_file = knowledge::DataFile::Turtle(archi_contents);

    knowledge.import(&dataset, togaf_file).await;
    knowledge.import(&dataset, archi_file).await;

    let architecture = architecture::DataBackedArchitectureService {
        dataset: &dataset,
        knowledge: &knowledge,
    };
    let components = architecture.components().await;
    println!("result: {:?}", components);

    let relations = architecture.relations().await;
    println!("relations: {:?}", relations);

    knowledge.delete(&dataset).await;

    let dot = graph(components, relations);

    let child = process::Command::new("dot")
        .arg("-Tsvg")
        .arg("-oout.svg")
        .stdin(process::Stdio::piped())
        .spawn()
        .expect("failed to execute dot");

    let mut outstdin = child.stdin.expect("failed to obtain stdin");
    outstdin.write_all(dot.as_bytes()).expect("failed to write");

    println!("graph: {}", dot);

    Ok(())
}
