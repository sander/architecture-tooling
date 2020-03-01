pub mod live;

use async_trait::async_trait;

#[derive(Debug)]
pub struct Dataset {
    name: String,
}

#[derive(Debug)]
pub enum DatasetCreationError {
    NameAlreadyTaken(Dataset),
    IO(String),
}

#[derive(Debug)]
pub enum DatasetDeletionError {
    DatasetNotFound,
    IO(String),
}

#[async_trait]
pub trait Service {
    async fn create(&self, name: &str) -> Result<Dataset, DatasetCreationError>;
    async fn delete(&self, dataset: Dataset) -> Result<(), DatasetDeletionError>;
}
