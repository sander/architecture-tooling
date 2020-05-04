#[cfg(test)]
mod tests {
    use crate::document::{Document, MarkdownDocument};
    use crate::knowledge::{get_labels, FusekiKnowledgeService, KnowledgeService};
    use tokio::{fs, io};

    async fn workflow() -> io::Result<()> {
        let client = reqwest::Client::new();
        let knowledge = FusekiKnowledgeService::new(&client, "http://localhost:3030/");
        let dataset = knowledge.create_dataset("architecture".to_string()).await;

        let contents = fs::read_to_string("test.md").await?;
        let doc = MarkdownDocument::from(&contents);
        let links = doc.get_outbound_links();

        let map = get_labels(&knowledge, &dataset, links).await;

        let render = doc.render_with_replaced_labels(map);

        println!("contents: {:?}", doc);
        // println!("links: {:?}", links);
        println!("render: {:?}", render);
        Ok(())
    }

    #[tokio::test]
    async fn run() {
        let _ = workflow().await;
    }
}
