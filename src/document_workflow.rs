#[cfg(test)]
mod tests {
    use crate::document::{Document, MarkdownDocument};
    use crate::knowledge::FusekiKnowledgeService;
    use futures::FutureExt;
    use tokio::{fs, io};

    async fn workflow() -> io::Result<()> {
        // let client = reqwest::Client::new();
        // let local = url::Url::parse("http://localhost:3030/").expect("parse error");
        // let knowledge = FusekiKnowledgeService {
        //     client: &client,
        //     base: local,
        // };

        let contents = fs::read_to_string("src/test.md").await?;
        let doc = MarkdownDocument::from(&contents);
        let links = doc.get_outbound_links();

        println!("contents: {:?}", doc);
        println!("links: {:?}", links);
        Ok(())
    }

    #[tokio::test]
    async fn run() {
        workflow().await;
        assert_eq!(1, 2);
    }
}
