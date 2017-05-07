use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::time;
use std::collections::HashMap;

use tempdir::TempDir;
use tokio_service::Service;
use futures::{future, Future, BoxFuture};
use {md5, base64};

use super::{SRV_DIRECTORY, MANIFEST_FILE};
use super::line::{LineIn,LineOut,Request};


pub struct Rsync {
    dir: PathBuf,
}

impl Rsync {
    pub fn new() -> io::Result<Self> {
        let tmpdir = TempDir::new(SRV_DIRECTORY)?.into_path();
        let manifest = tmpdir.join(Path::new(MANIFEST_FILE));
        File::create(manifest)?; 
        Ok(Rsync {
            dir: tmpdir,
        })
    }
    fn get_s(&self, p: &Path) -> io::Result<String> {
        let path = self.dir.join(p);
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(s)
    }
    fn get_b64(&self, p: &Path) -> io::Result<String> {
        let ptxt = self.get_s(p)?;
        Ok(base64::encode(&ptxt))
    }
    fn put_b64(&self, p: &Path, b64: &str) -> io::Result<()> {
        // decode text
        let bytes = base64::decode(b64)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid base64"))?;
        let text = String::from_utf8_lossy(&bytes);
        // write text to file
        let path = self.dir.join(p);
        let mut f = File::create(path)?;
        f.write_all(text.as_bytes())?;
        // update entry in manifest
        let mut contents = self.get_contents()?;
        let name = p.file_name()
            .expect("Tried to write to a non-file")
            .to_string_lossy()
            .into_owned();
        contents.insert(name, &text);
        self.put_contents(contents)
    }
    fn query(&self, p: &Path) -> io::Result<String> {
        // returns seconds between Jan 1 1970 and p's last modification
        let path = self.dir.join(p);
        let f = File::open(path)?;
        let md = f.metadata()?;
        let last_modified = md.modified()?;
        let age = last_modified.duration_since(time::UNIX_EPOCH).unwrap();
        let seconds = age.as_secs();
        Ok(seconds.to_string())
    }
    fn get_contents(&self) -> io::Result<Contents> {
        let s = self.get_s(Path::new(MANIFEST_FILE))?;
        Contents::from_string(&s).ok_or(io::Error::new(io::ErrorKind::Other, 
                                                       "bad manifest"))
    }
    fn put_contents(&self, c: Contents) -> io::Result<()> {
        let path = self.dir.join(Path::new(MANIFEST_FILE));
        let mut f = File::create(path)?;
        let s = c.to_string();
        f.write_all(s.as_bytes())
    }
}

impl Service for Rsync {
    type Request = LineIn;
    type Response = LineOut;
    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        println!("Request:  {:?}", req);
        let resp = match req {
            Request::Contents => self.get_s(Path::new(MANIFEST_FILE)),
            Request::Query(pb) => self.query(&pb),
            Request::Get(pb) => self.get_b64(&pb),
            Request::Put { file, text} => 
                self.put_b64(&file, &text).map(|()| "ACK".to_string()),
            Request::Invalid => 
                Ok(String::from("ERROR: try command contents|query|get|put"))
        };
        println!("Result:   {:?}\n", resp);
        future::result(resp).boxed()
    }
}

// Contents representation

struct Contents(HashMap<String,String>);    // maps filename to hash

impl Contents {
    fn insert(&mut self, name: String, text: &str) {
        let digest = md5::compute(text.as_bytes());
        self.0.insert(name, format!("{:x}", digest));
    }
    fn from_string(s: &str) -> Option<Self> {
        let mut v: HashMap<String,String> = HashMap::new();
        for line in s.trim().split('\n') {
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, "    ");
            let h = parts.nth(0);
            let f = parts.nth(0);
            if let (Some(hash),Some(name)) = (h, f) {
                v.insert(name.to_string(), hash.to_string());
            } else {
                return None;
            }
        }
        Some(Contents(v))
    }
    fn to_string(&self) -> String {
        let mut s = String::new();
        for (name,hash) in &self.0 {
            let f = format!("{}    {}\n", hash, name);
            s.push_str(&f);
        }
        s
    }
}
