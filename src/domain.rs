use select::{document::Document, predicate::Name};
use std::collections::HashSet;

const PROTOCOL: &str = "https://";

/// append protocol on origin for http client
pub fn check_protocol(org: String) -> String {
    if !org.contains(PROTOCOL) {
        let mut u = String::from(PROTOCOL);
        u.push_str(&org);
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
    for link in links {
        if link.starts_with('/') {
            let noramlized_url = format!("https://{}{}", base, &link);
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
    use url::Url;

    #[test]
    fn test_check_protocol() {
        let arg_str = String::from("blog.com");
        assert_eq!(check_protocol(arg_str), String::from("https://blog.com"));

        let arg_str_2 = String::from("https://anotherblog.com");
        assert_eq!(
            check_protocol(arg_str_2),
            String::from("https://anotherblog.com")
        );
    }

    #[tokio::test]
    async fn test_process_domain_links() {
        let doc_string = "<div><ul><li><a href='https://www.linkedin.com/company/github'></a></li><li><a href='/github/path_2'></a></li></ul>";
        let r = process_domain_links(doc_string, "sombase.com").await;
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_parse_links() {
        let mut set = HashSet::new();
        set.insert("example-base.com".to_owned());
        set.insert("/path/path".to_owned());
        set.insert("https://external.org/page-1".to_owned());
        set.insert("https://example-base.com/some-path".to_owned());

        let results = parse_links("example-base.com", set);

        let filtered = results
            .into_iter()
            .map(|s| check_protocol(s))
            .filter(|it| Url::parse(it).is_ok())
            .collect::<Vec<String>>();

        assert_eq!(filtered.len(), 3);
    }
}
