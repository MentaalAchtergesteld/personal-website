use std::{collections::HashMap, u8};

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => result.push(b' '),
            b'%' if i+2 < bytes.len() => {
                let hex = &s[i+1..i+3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    result.push(v);
                    i+=3;
                } else {
                    i+=1;
                }
            },
            _ => {
                result.push(bytes[i]);
                i+=1;
            }
        }
    }

    String::from_utf8_lossy(&result).into_owned()
}

pub fn parse_urlencoded(body: &str) -> HashMap<String, String> {
    body.split("&").filter_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(key), Some(value)) => {
                let key = url_decode(key);
                let value = url_decode(value);

                Some((key, value))
            },
            _ => None
        }
    }).collect()
}
