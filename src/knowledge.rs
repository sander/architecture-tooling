use async_trait::async_trait;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Collection of default graph and one or more named graphs.
#[derive(Debug)]
pub struct Dataset {
    name: String,
}

/// A resource, as in RDF, identified by IRI.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Resource(String);

impl From<String> for Resource {
    fn from(s: String) -> Self {
        Resource(s)
    }
}

impl From<&str> for Resource {
    fn from(s: &str) -> Self {
        Resource(s.to_string())
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub vars: Vec<String>,
    pub bindings: Vec<HashMap<String, rdf::node::Node>>,
}

#[derive(Debug)]
pub struct Graph {
    jsonld_representation: String,
}

/// Provides knowledge management.
#[async_trait]
pub trait KnowledgeService {
    /// Creates a dataset for importing and querying.
    async fn create_dataset(&self, name: String) -> Dataset;

    /// Performs a SPARQL query.
    async fn select(&self, dataset: &Dataset, query: &str) -> QueryResult;

    async fn describe(&self, dataset: &Dataset, query: &str) -> Graph;
}

pub struct FusekiKnowledgeService<'a> {
    client: &'a reqwest::Client,
    base: url::Url,
}

impl FusekiKnowledgeService<'_> {
    pub fn new<'a>(client: &'a reqwest::Client, base_url: &'a str) -> impl KnowledgeService + 'a {
        let local = url::Url::parse(base_url).expect("parse error");
        FusekiKnowledgeService {
            client: &client,
            base: local,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct QueryResponseHead {
    vars: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
enum QueryResponseValue {
    uri { value: String },
    literal { value: String },
}

impl QueryResponseValue {
    fn to_node(&self) -> rdf::node::Node {
        match self {
            QueryResponseValue::uri { value } => rdf::node::Node::UriNode {
                uri: rdf::uri::Uri::new(value.to_string()),
            },
            QueryResponseValue::literal { value } => rdf::node::Node::LiteralNode {
                literal: value.to_string(),
                data_type: None,
                language: None,
            },
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct QueryResponseResults {
    bindings: Vec<HashMap<String, QueryResponseValue>>,
}

#[derive(Deserialize, Debug)]
pub struct QueryResponse {
    head: QueryResponseHead,
    results: QueryResponseResults,
}

#[async_trait]
impl KnowledgeService for FusekiKnowledgeService<'_> {
    async fn create_dataset(&self, name: String) -> Dataset {
        match self
            .client
            .post(self.base.join("/$/datasets").unwrap())
            .form(&[("dbName", &name), ("dbType", &"mem".to_string())])
            .send()
            .await
            .unwrap()
            .status()
        {
            reqwest::StatusCode::CONFLICT | reqwest::StatusCode::OK => Dataset { name },
            _ => panic!("Error creating dataset {}.", name),
        }
    }

    async fn select(&self, dataset: &Dataset, query: &str) -> QueryResult {
        let form = [("query", query)];
        let path = self.base.join(&dataset.name).unwrap();
        let response = self.client.post(path).form(&form).send().await.unwrap();
        let status = response.status();
        let response = response.json::<QueryResponse>().await.unwrap();
        match status {
            reqwest::StatusCode::OK => QueryResult {
                vars: response.head.vars,
                bindings: response
                    .results
                    .bindings
                    .iter()
                    .map(|binding| {
                        binding
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_node()))
                            .collect::<HashMap<String, rdf::node::Node>>()
                    })
                    .collect::<Vec<_>>(),
            },
            code => panic!("Unexpected status {}.", code),
        }
    }

    async fn describe(&self, dataset: &Dataset, query: &str) -> Graph {
        let form = [("query", query)];
        let path = self.base.join(&dataset.name).unwrap();
        let response = self
            .client
            .post(path)
            .header(reqwest::header::ACCEPT, "application/ld+json")
            .form(&form)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let response = response.text().await.unwrap();
        match status {
            reqwest::StatusCode::OK => Graph {
                jsonld_representation: response,
            },
            code => panic!("Unexpected status {}.", code),
        }
    }
}

async fn get_label(
    service: &impl KnowledgeService,
    dataset: &Dataset,
    resource: &Resource,
) -> Option<String> {
    let query = "prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?label
WHERE {
  GRAPH ?g { <$RESOURCE> rdfs:label ?label }
}
LIMIT 25"
        .replace("$RESOURCE", &resource.0);
    let results = service.select(dataset, &query).await;
    let key = "label";

    assert_eq!(results.vars, [key]);

    results
        .bindings
        .get(0)
        .and_then(|b| b.get(key))
        .and_then(|n| match n {
            rdf::node::Node::LiteralNode {
                literal: label,
                data_type: _,
                language: _,
            } => Some(label.to_string()),
            _ => None,
        })
}

pub async fn get_labels(
    service: &impl KnowledgeService,
    dataset: &Dataset,
    resources: HashSet<Resource>,
) -> HashMap<Resource, String> {
    let mut result = HashMap::new();
    for resource in resources {
        match get_label(service, dataset, &resource).await {
            Some(label) => {
                result.insert(resource, label);
            }
            None => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::knowledge::{get_labels, FusekiKnowledgeService, KnowledgeService, Resource};
    use std::collections::HashSet;

    #[tokio::test]
    async fn can_get_labels() {
        let client = reqwest::Client::new();
        let knowledge = FusekiKnowledgeService::new(&client, "http://localhost:3030/");
        let name = "architecture";
        let dataset = knowledge.create_dataset(name.to_string()).await;

        let mut resources = HashSet::new();
        let key = Resource::from(
            "http://localhost:3030/architecture/data?graph=architecture.ttl#architecture-application-service",
        );
        resources.insert(key.clone());

        let labels = get_labels(&knowledge, &dataset, resources).await;

        assert_eq!(labels.len(), 1);
        assert_eq!(
            labels.get(&key),
            Some(&"architecture application service".to_string())
        );
        println!("labels: {:?}", labels);
    }
}
