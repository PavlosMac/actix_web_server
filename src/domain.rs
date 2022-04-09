use select::{document::Document, predicate::Name};
use std::collections::HashSet;

pub async fn process_domain_links(
    res: &str,
    org: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let links: HashSet<String> = Document::from(res)
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|n| n.to_owned())
        .collect::<HashSet<String>>();
    let l = parse_links(org.as_str(), links);

    Ok(l)
}

// /// loop links from domain, if link is path only, append domain, if link base is substring add as indexable
fn parse_links(base: &str, links: HashSet<String>) -> Vec<String> {
    println!("parse links called --- {}", &base);
    let mut indexables = Vec::new();
    for link in links {
        if link.starts_with('/') {
            let full_u = format!("https://{}{}", base, &link);
            indexables.push(full_u);
        }
        if link.contains(&base) {
            indexables.push(link)
        }
    }
    indexables
}

// /// append protocol on origin for http client
// fn check_protocol(org: String) -> String {
//     if !org.contains(PROTOCOL) {
//         let mut u = String::from(PROTOCOL);
//         u.push_str(&org.to_string());
//         return u;
//     }
//     org
// }
