use pulldown_cmark::{html, CowStr, Event, Parser, Tag};
use std::collections::{HashMap, HashSet};

/// A resource, as in RDF, identified by IRI.
#[derive(Hash, PartialEq, Eq, Debug)]
pub struct Resource(String);

#[derive(Debug)]
pub struct MarkdownDocument<'a>(pub &'a str);

pub trait Document {
    fn get_outbound_links(&self) -> HashSet<Resource>;
    fn render_with_replaced_labels(&self, labels: HashMap<Resource, String>) -> String;
}

impl<'a> From<&'a String> for MarkdownDocument<'a> {
    fn from(s: &'a String) -> Self {
        MarkdownDocument(s)
    }
}

impl Document for MarkdownDocument<'_> {
    fn get_outbound_links(&self) -> HashSet<Resource> {
        let mut links = HashSet::new();
        let parser = Parser::new(self.0).flat_map(|event| match event {
            Event::End(Tag::Link(_, destination_url, _)) => {
                links.insert(Resource(destination_url.to_string()));
                vec![]
            }
            _ => vec![],
        });
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        links
    }

    fn render_with_replaced_labels(&self, labels: HashMap<Resource, String>) -> String {
        let mut current_label = None;
        let parser = Parser::new(self.0).flat_map(|event| match (event, current_label) {
            (Event::Start(Tag::Link(link_type, destination_url, title)), _) => {
                current_label = labels.get(&Resource(destination_url.to_string()));
                vec![Event::Start(Tag::Link(link_type, destination_url, title))]
            }
            (Event::End(t @ Tag::Link(_, _, _)), Some(label)) => {
                current_label = None;
                vec![Event::Text(CowStr::from(label.to_string())), Event::End(t)]
            }
            (_, Some(_)) => vec![],
            (e, None) => vec![e],
        });

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }
}

#[cfg(test)]
mod tests {
    use crate::document::{Document, MarkdownDocument, Resource};
    use std::collections::HashMap;

    #[test]
    fn extracts_outbound_links() {
        let doc = MarkdownDocument("[Hello] [world](#foo).\n\n[Hello]: http://example.com/");
        let links = doc.get_outbound_links();

        assert_eq!(links.len(), 2);
        assert!(links.contains(&Resource("http://example.com/".to_string())));
        assert!(links.contains(&Resource("#foo".to_string())));
    }

    #[test]
    fn replaces_found_labels() {
        let doc = MarkdownDocument("[Hello] [world](#foo).\n\n[Hello]: http://example.com/");

        let mut labels = HashMap::new();
        labels.insert(Resource("#foo".to_string()), "foo".to_string());
        assert_eq!(
            "<p><a href=\"http://example.com/\">Hello</a> <a href=\"#foo\">foo</a>.</p>\n",
            doc.render_with_replaced_labels(labels)
        );
    }
}
