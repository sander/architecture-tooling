use uuid::Uuid;

use async_trait::async_trait;

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

#[async_trait]
pub trait KnowledgeService {
    async fn create_temporary_dataset(&self) -> Dataset;
    async fn import(&self, dataset: &Dataset, file: DataFile);
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
}
