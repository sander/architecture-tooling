use super::{Dataset, DatasetCreationError, DatasetDeletionError, Service};
use async_trait::async_trait;

pub struct FusekiKnowledgeService {
    client: reqwest::Client,
    path: url::Url,
}

impl FusekiKnowledgeService {
    pub fn new(client: reqwest::Client, path: url::Url) -> FusekiKnowledgeService {
        FusekiKnowledgeService { client, path }
    }
}

#[async_trait]
impl Service for FusekiKnowledgeService {
    async fn create(&self, name: &str) -> Result<Dataset, DatasetCreationError> {
        match self
            .client
            .post(self.path.join("/$/datasets").unwrap())
            .form(&[("dbName", name), ("dbType", "mem")])
            .send()
            .await
            .map_err(|e| DatasetCreationError::IO(e.to_string()))?
            .status()
        {
            reqwest::StatusCode::CONFLICT => Err(DatasetCreationError::NameAlreadyTaken(Dataset {
                name: name.to_owned(),
            })),
            _ => Ok(Dataset {
                name: name.to_owned(),
            }),
        }
    }

    async fn delete(&self, dataset: Dataset) -> Result<(), DatasetDeletionError> {
        match self
            .client
            .delete(
                self.path
                    .join("/$/datasets/")
                    .unwrap()
                    .join(&dataset.name)
                    .unwrap(),
            )
            .send()
            .await
            .map_err(|e| DatasetDeletionError::IO(e.to_string()))?
            .status()
        {
            reqwest::StatusCode::CONFLICT => Err(DatasetDeletionError::DatasetNotFound),
            _ => Ok(()),
        }
    }
}
