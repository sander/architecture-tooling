use super::knowledge;
use async_trait::async_trait;

#[derive(Debug)]
pub enum ComponentKind {
    BusinessService,
    Function,
    InformationSystemService,
}

#[derive(Debug)]
pub struct Component {
    pub label: String,
    pub description: Option<String>,
    pub kind: ComponentKind,
}

#[async_trait]
pub trait ArchitectureService {
    async fn components(&self) -> Vec<Component>;
}

pub struct DataBackedArchitectureService<'a, K: knowledge::KnowledgeService + 'a> {
    pub dataset: &'a knowledge::Dataset,
    pub knowledge: &'a K,
}

#[async_trait]
impl<'a, K: knowledge::KnowledgeService + 'a + std::marker::Sync> ArchitectureService
    for DataBackedArchitectureService<'a, K>
{
    async fn components(&self) -> Vec<Component> {
        let query = std::include_str!("components.sparql");
        let result = &self.knowledge.query(&self.dataset, query).await;

        assert_eq!(result.vars, ["component", "label", "description", "kind"]);

        result
            .bindings
            .iter()
            .map(|record| {
                Component {
                    label: match record.get("label") {
                        Some(rdf::node::Node::LiteralNode {
                            literal: label,
                            data_type: _,
                            language: _,
                        }) => label.to_string(),
                        _ => panic!("Unexpected label"),
                    },
                    description: match record.get("description") {
                        Some(rdf::node::Node::LiteralNode {
                            literal: description,
                            data_type: _,
                            language: _,
                        }) => Some(description.to_string()),
                        None => None,
                        d => panic!("Unexpected description {:?}", d),
                    },
                    kind: match record.get("kind") {
                        Some(rdf::node::Node::UriNode { uri }) => match uri.to_string().as_str() {
                            "http://www.semanticweb.org/ontologies/2010/0/OntologyTOGAFContentMetamodel.owl#BusinessService" => ComponentKind::BusinessService,
                            "http://www.semanticweb.org/ontologies/2010/0/OntologyTOGAFContentMetamodel.owl#InformationSystemService" => ComponentKind::InformationSystemService,
                            "http://www.semanticweb.org/ontologies/2010/0/OntologyTOGAFContentMetamodel.owl#Function" => ComponentKind::Function,
                            kind => panic!("Unknown kind {}.", kind)
                        },
                        _ => panic!("Unexpected kind"),
                    },
                }
            })
            .collect::<Vec<_>>()
    }
}
