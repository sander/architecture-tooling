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
    label: String,
    description: Option<String>,
    kind: ComponentKind,
}

#[async_trait]
pub trait ArchitectureService {
    async fn components(&self) -> Vec<Component>;
}

pub struct DataBackedArchitectureService<'a, K: knowledge::KnowledgeService + 'a> {
    dataset: &'a knowledge::Dataset,
    knowledge: &'a K,
}

impl<'a, K: knowledge::KnowledgeService + 'a> DataBackedArchitectureService<'a, K> {
    pub fn new(
        dataset: &'a knowledge::Dataset,
        knowledge: &'a K,
    ) -> DataBackedArchitectureService<'a, K> {
        DataBackedArchitectureService { dataset, knowledge }
    }
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
                    kind: ComponentKind::BusinessService, // TODO
                                                          //   kind: match record.get("kind") {
                                                          //         Some(rdf::node::Node::UriNode {
                                                          //             uri
                                                          //         }) if uri == &rdf::uri::Uri { uri : "http://www.semanticweb.org/ontologies/2010/0/OntologyTOGAFContentMetamodel.owl#BusinessService".to_string() }  => ComponentKind::BusinessService,
                                                          //         _ => panic!("Unexpected kind"),
                }
            })
            .collect::<Vec<_>>()
    }
}
