use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Dataset {
    name: String,
}

#[derive(Debug)]
pub enum DataFile {
    Turtle(Vec<u8>),
    RdfXml(Vec<u8>),
}

impl DataFile {
    fn multipart(self) -> reqwest::multipart::Part {
        match self {
            DataFile::Turtle(contents) => reqwest::multipart::Part::bytes(contents)
                .file_name("file.ttl")
                .mime_str("text/turtle")
                .unwrap(),
            DataFile::RdfXml(contents) => reqwest::multipart::Part::bytes(contents)
                .file_name("file.xml")
                .mime_str("text/xml")
                .unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub vars: Vec<String>,
    pub bindings: Vec<HashMap<String, rdf::node::Node>>,
}

#[async_trait]
pub trait KnowledgeService {
    async fn create_temporary_dataset(&self) -> Dataset;
    async fn import(&self, dataset: &Dataset, file: DataFile);
    async fn delete(&self, dataset: &Dataset);
    async fn query(&self, dataset: &Dataset, query: &str) -> QueryResult;
}

pub struct FusekiKnowledgeService<'a> {
    client: &'a reqwest::Client,
    base: url::Url,
}

impl FusekiKnowledgeService<'_> {
    pub fn new(client: &reqwest::Client, base: url::Url) -> FusekiKnowledgeService {
        FusekiKnowledgeService { client, base }
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
    async fn create_temporary_dataset(&self) -> Dataset {
        let name = Uuid::new_v4().to_hyphenated().to_string();
        match self
            .client
            .post(self.base.join("/$/datasets").unwrap())
            .form(&[("dbName", &name), ("dbType", &"mem".to_string())])
            .send()
            .await
            .unwrap()
            .status()
        {
            reqwest::StatusCode::CONFLICT => panic!("Dataset named {} already exists.", name),
            _ => Dataset { name },
        }
    }

    async fn import(&self, dataset: &Dataset, file: DataFile) {
        let form = reqwest::multipart::Form::new().part("files[]", file.multipart());
        let path = self.base.join(&format!("/{}/data", &dataset.name)).unwrap();
        let response = self.client.post(path).multipart(form).send().await.unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        match status {
            reqwest::StatusCode::OK => (),
            code => panic!("Unexpected status {} with message {}.", code, body),
        };
    }

    async fn delete(&self, dataset: &Dataset) {
        let path = self
            .base
            .join("/$/datasets/")
            .unwrap()
            .join(&dataset.name)
            .unwrap();
        match self.client.delete(path).send().await.unwrap().status() {
            reqwest::StatusCode::OK => (),
            code => panic!("Unexpected status {}.", code),
        }
    }

    async fn query(&self, dataset: &Dataset, query: &str) -> QueryResult {
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
}
