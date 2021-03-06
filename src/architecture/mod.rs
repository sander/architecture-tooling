use super::knowledge;
use crate::knowledge::Graph;
use async_trait::async_trait;

#[derive(Debug)]
pub enum ComponentKind {
    BusinessService,
    Function,
    InformationSystemService,
    Process,
}

#[derive(Debug)]
pub struct ComponentId {
    pub value: String,
}

/// An architecture component. Of any level.
#[derive(Debug)]
pub struct Component {
    pub id: ComponentId,
    pub label: String,
    pub description: Option<String>,
    pub kind: ComponentKind,
}

#[derive(Debug)]
pub struct Relation {
    pub from: ComponentId,
    pub to: ComponentId,
    pub label: String,
}

#[derive(Debug)]
pub struct Visualization {
    pub graphviz_content: String,
}

#[async_trait]
pub trait ArchitectureService {
    async fn components(&self) -> Vec<Component>;
    async fn relations(&self) -> Vec<Relation>;
    async fn component_ids(&self) -> Vec<ComponentId>;
    async fn describe(&self, component_id: &ComponentId) -> super::knowledge::Graph;
}

pub fn visualization(components: Vec<Component>, relations: Vec<Relation>) -> Visualization {
    let mut s = "digraph components_relations {\n".to_string();

    for c in components.iter() {
        s.push('"');
        s.push_str(c.id.value.replace('"', "\\\"").as_str());
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
        s.push_str(r.from.value.replace('"', "\\\"").as_str());
        s.push_str("\" -> \"");
        s.push_str(r.to.value.replace('"', "\\\"").as_str());
        s.push_str("\" [label=\"");
        s.push_str(r.label.replace('"', "\\\"").as_str());
        s.push_str("\"]\n");
    }

    s.push_str("}");

    Visualization {
        graphviz_content: s,
    }
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
        let query: &str = std::include_str!("components.sparql");
        let result = &self.knowledge.select(&self.dataset, query).await;

        assert_eq!(result.vars, ["component", "label", "description", "kind"]);

        result
            .bindings
            .iter()
            .map(|record| {
                Component {
                    id: match record.get("component") {
                        Some(rdf::node::Node::UriNode { uri }) => ComponentId { value: uri.to_string().to_string() },
                        c => panic!("Unexpected component {:?}", c),
                    },
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
                            "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#BusinessService" => ComponentKind::BusinessService,
                            "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#InformationSystemService" => ComponentKind::InformationSystemService,
                            "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#Function" => ComponentKind::Function,
                            "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#Process" => ComponentKind::Process,
                            kind => panic!("Unknown kind {}.", kind)
                        },
                        _ => panic!("Unexpected kind"),
                    },
                }
            })
            .collect::<Vec<_>>()
    }

    async fn component_ids(&self) -> Vec<ComponentId> {
        let query: &str = std::include_str!("component_ids.sparql");
        let result = &self.knowledge.select(&self.dataset, query).await;

        assert_eq!(result.vars, ["component"]);

        result
            .bindings
            .iter()
            .map(|record| ComponentId {
                value: match record.get("component") {
                    Some(rdf::node::Node::UriNode { uri }) => uri.to_string().to_string(),
                    c => panic!("Unexpected id {:?}", c),
                },
            })
            .collect::<Vec<_>>()
    }

    async fn relations(&self) -> Vec<Relation> {
        let query: &str = std::include_str!("relations.sparql");
        let result = &self.knowledge.select(&self.dataset, query).await;

        assert_eq!(result.vars, ["from", "label", "to"]);

        result
            .bindings
            .iter()
            .map(|record| Relation {
                from: match record.get("from") {
                    Some(rdf::node::Node::UriNode { uri }) => ComponentId { value: uri.to_string().to_string() },
                    _ => panic!("Unexpected from"),
                },
                label: match record.get("label") {
                    Some(rdf::node::Node::UriNode { uri }) => match uri.to_string().as_str() {
                        "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#informationSystemServiceRealizesBusinessService" => "realizes".to_string(),
                        "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#businessServiceProvidesGovernedInterfaceToAccessFunction" => "provides governed interface to access".to_string(),
                        "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#processDecomposesAndOrOrchestratesFunction" => "orchestrates and/or decomposes".to_string(),
                        "http://www.semanticweb.org/ontologies/2020/4/OntologyTOGAFContentMetamodel.owl#functionIsRealizedByAndOrSupportsProcess" => "supports or is realized by".to_string(),
                        url => panic!("Unknown url {}.", url)
                    },
                    _ => panic!("Unexpected label"),
                },
                to: match record.get("to") {
                    Some(rdf::node::Node::UriNode { uri }) => ComponentId { value: uri.to_string().to_string() },
                    _ => panic!("Unexpected to"),
                },
            })
            .collect::<Vec<_>>()
    }

    async fn describe(&self, component_id: &ComponentId) -> Graph {
        let template: &str = std::include_str!("describe.sparql");
        let query = template.replace("$ID", component_id.value.as_str());
        let result = self.knowledge.describe(&self.dataset, query.as_str()).await;
        result
    }
}
