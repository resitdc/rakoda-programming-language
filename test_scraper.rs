use std::sync::Arc;
use scraper::{Html, Selector, ElementRef};

fn test_id(id: ego_tree::NodeId) {}

fn main() {
    let document = Arc::new(Html::parse_document("<html></html>"));
    let root_id = document.tree.root().id();
    test_id(root_id);
}
