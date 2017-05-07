use std::path::PathBuf;

pub type LineIn = Request;
pub type LineOut = String;

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Contents,
    Query(PathBuf),
    Get(PathBuf), 
    Put {
        file: PathBuf,
        text: String,
    },
    Invalid,
}

impl Request {
    pub fn parse(s: &str) -> Self {
        // must (only) return a resposne
        let mut tokens = s.split_whitespace();
        let command = tokens.nth(0);
        let filename = tokens.nth(0);
        let filebody = tokens.nth(0);   // base64: no whitespace
        match (command, filename, filebody) {
            (Some("contents"),None, None) =>    Request::Contents,
            (Some("query"), Some(f),None) =>    Request::Query(PathBuf::from(f)),
            (Some("get"),   Some(f),None) =>    Request::Get(PathBuf::from(f)),
            (Some("put"),   Some(f),Some(b)) => Request::Put {
                file: PathBuf::from(f),
                text: b.to_string(),
            },
            _ => Request::Invalid,
        }
    }
}

