
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::env;
use std::path::{PathBuf, Path};
use std::fs::{File, OpenOptions, DirBuilder};
use std::time::SystemTime;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


const FOLDER_NAME: &'static str = "contents";
const MANIFEST_TITLE: &'static str = ".4220_file_list.txt";

fn handle_client(mut stream: TcpStream) {
    let mut s = String::new();
    let a = stream.read_to_string(&mut s);
}

fn build_file_path(p: &Path) -> io::Result<PathBuf> {
    let mut here = env::current_dir()?;
    here.push(FOLDER_NAME);
    here.push(p);
    Ok(here)
}

fn contents() -> io::Result<String> {
    let manifest = build_file_path(Path::new(MANIFEST_TITLE))?;
    let mut f = File::open(manifest)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn query(filename: &Path) -> io::Result<SystemTime> {
    let path = build_file_path(filename)?;
    let f = File::open(path)?;
    let md = f.metadata()?;
    md.modified()
}

fn get(filename: &Path) -> io::Result<String> {
    let path = build_file_path(filename)?;
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn put(filename: &Path, contents: String) -> io::Result<()> {
    let path = build_file_path(filename)?;
    let mut f = File::create(path)?;
    f.write_all(contents.as_bytes())
}

fn hash(filename: &Path) -> io::Result<u64> {
    // NOTE: this uses SipHasher, the only hash algo in Rust's std lib
    // it would be trivial to use the md5 crate, but this requires no crates
    // todo fix? idk
    let text = get(filename)?;
    let mut s = DefaultHasher::new();
    text.hash(&mut s);
    Ok(s.finish())
}



fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    //let mut stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    //let _ = stream.write(&[1]);
    //let _ = stream.read(&mut [0; 128]); 
    for stream in listener.incoming() {
        handle_client(stream.expect("Found invalid stream"));
    }
}
