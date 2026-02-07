use std::collections::HashMap;

pub mod cache;

pub fn parse_query(url: &str) -> HashMap<String, String> {
    let query_str = match url.split_once("?") {
        Some((_, qs)) => qs,
        None => return HashMap::new()
    };

    query_str.split("&")
        .filter_map(|pair| {
            let (key, value) = pair.split_once("=")?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

