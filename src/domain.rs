use select::{document::Document, predicate::Name};
use std::collections::HashSet;

pub async fn process_domain_links(res: &str, org: Url) -> Vec<String> {
    let links: HashSet<String> = Document::from(res)
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|n| n.to_owned())
        .collect::<HashSet<String>>();
    parse_links(org, links)
}

// /// loop links from domain, if link is path only, append domain, if link base is substring add as indexable
fn parse_links(base: &str, links: HashSet<String>) -> Vec<String> {
    let mut indexables = Vec::new();
    for mut link in links {
        if link.starts_with('/') {
            link.remove(0);
            let noramlized_url = format!("{}{}", base, &link);
            indexables.push(noramlized_url);
            continue;
        }
        if link.contains("https://") {
            indexables.push(link)
        }
    }
    indexables
}
