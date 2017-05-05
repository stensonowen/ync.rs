
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::env;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::time::SystemTime;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


// TODO (?): use rust temp dir?
const FOLDER_NAME: &'static str = "tmp_contents";
const MANIFEST_TITLE: &'static str = ".4220_file_list.txt";

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    //let mut s = String::new();
    //stream.read_to_string(&mut s)?;
    let mut b = [b' ';32];
    //let mut b = Vec::with_capacity(32);
    //stream.read_exact(&mut b)?;
    stream.read(&mut b)?;
    //stream.read(&mut b)?;
    //println!("{}", b.len());
    //let s = String::from_utf8(b).expect("Bad utf");
    let s = String::from_utf8_lossy(&b).to_owned();
    //println!("{}", s.len());
    //println!("`{}`", s);
    //println!("Received: `{}`", s);
    
    // case sensitivity?
    //match s.splitn(1, " ").nth(0).map(|t| t.to_lowercase()) {
    let mut tokens = s.splitn(3, " ");
    let tokall: Vec<_> = tokens.clone().collect();
    println!("{:?}", tokall);
    let command = tokens.nth(0);
    let file = tokens.nth(0).map(|b| Path::new(b.trim()));
    let body = tokens.nth(0);
    println!("command: `{:?}`", command);
    //let fn_error_msg = String::from("ERROR: expected a filename");
    let response: String = match command {
        Some("contents") => contents()?,
        Some("query") => match file {
            Some(filename) => format!("{:?}", query(filename)?),
            None => "ERROR: try `query <filename>`".to_string(),
        },
        Some("get") => match file {
            Some(filename) => get(filename)?,
            None => "ERROR: try `get <filename>`".to_string(),
        },
        //Some("get_") => file.map_or_else(|f| get(f)?, || fn_error_msg),
        Some("put") => match (file,body) {
            (Some(f),Some(b)) => {put(f, b)?; String::from("thanks")},
            (Some(_),None) => "ERROR: missing `body`".to_string(),
            (None,Some(_)) => "ERROR: missing `filename`".to_string(),
            _ => "ERROR: try `put <filename> <body>`".to_string(),
        },
        _ => "ERROR: try command contents|query|get|put".to_string()
    };
    println!("Responding `{}`", response);
    stream.write_all(response.as_bytes())?;
    Ok(())
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
//fn query(filename: &str) -> io::Result<SystemTime> {
    let path = build_file_path(filename)?;
    let f = File::open(path)?;
    let md = f.metadata()?;
    md.modified()
}

fn get(filename: &Path) -> io::Result<String> {
//fn get(filename: &str) -> io::Result<String> {
    let path = build_file_path(filename)?;
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn put(filename: &Path, contents: &str) -> io::Result<()> {
    let path = build_file_path(filename)?;
    let mut f = File::create(path)?;
    f.write_all(contents.as_bytes())
}

fn hash(filename: &Path) -> io::Result<u64> {
    // NOTE: this uses the hash algorithm used by rust's hashmap
    //  because it's the only hash algorithm in the Rust std lib
    // it would be trivial to use the md5 crate, but this requires no crates 
    // todo fix? idk
    let text = get(filename)?;
    let mut s = DefaultHasher::new();
    text.hash(&mut s);
    Ok(s.finish())
}



fn main() {
    let mode = env::args().nth(1).expect("USAGE: ./sync client|server");

    if mode == "server" {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        //let _ = stream.write(&[1]);
        //let _ = stream.read(&mut [0; 128]); 
        for stream in listener.incoming() {
            handle_client(stream.expect("Found invalid stream")).unwrap();
        }
    } else if mode == "client" {
        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        //stream.write("contents".as_bytes()).unwrap();
        stream.write("get foo.txt".as_bytes()).unwrap();
        let mut s = String::new();
        stream.read_to_string(&mut s).unwrap();
        println!("`{}`", s);
    } else {
        panic!("Invalid mode string; it must be `client` or `server`");
    }
}
