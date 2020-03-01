use async_trait::async_trait;

pub struct Diagram {}

#[async_trait]
pub trait Service {
    async fn system_context_diagram(&self) -> Diagram;
}
