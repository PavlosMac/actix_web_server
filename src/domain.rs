use select::{document::Document, predicate::Name};
use std::collections::HashSet;

const PROTOCOL: &str = "https://";

/// append protocol on origin for http client
pub fn check_protocol(org: String) -> String {
    if !org.contains(PROTOCOL) {
        let mut u = String::from(PROTOCOL);
        u.push_str(&org.to_string());
        return u;
    }
    org
}

pub async fn process_domain_links(res: &str, org: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    // use httpmock::prelude::*;

    #[test]
    fn test_check_protocol() {
        let arg_str = String::from("blog.com");

        assert_eq!(check_protocol(arg_str), String::from("https://blog.com"));
    }

    // #[test]
    // fn test_check_protocol() {
    //     let arg_tes1 = String::from("https://blog.com");

    //     assert_eq!(check_protocol(String::from("https://blog.com")), arg_tes1);
    //     assert_eq!(check_protocol(String::from("blog.com")), arg_tes1);
    // }

    // #[test]
    // fn test_parse_links() {
    //     let mut set = HashSet::new();
    //     set.insert("example-base.com".to_owned());
    //     set.insert("/path/path".to_owned());
    //     set.insert("/example/path".to_owned());
    //     set.insert("https://example-base.com".to_owned());
    //     let results = parse_links("example-base.com".to_owned(), set);
    //     let filtered = results
    //         .into_iter()
    //         .map(|s| check_protocol(s))
    //         .filter(|it| {
    //             let r = Regex::new(r"^(https)://+example-base.com+([a-zA-Z0-9\\/]*)$");
    //             r.unwrap().is_match(it)
    //         })
    //         .collect::<Vec<String>>();

    //     assert_eq!(filtered.len(), 4);
    // }
}
