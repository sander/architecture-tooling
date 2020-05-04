#[cfg(test)]
mod tests {
    use crate::document::{Document, MarkdownDocument};
    use futures::FutureExt;
    use tokio::{fs, io};

    async fn workflow() -> io::Result<()> {
        let contents = fs::read_to_string("test.md").await?;
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
