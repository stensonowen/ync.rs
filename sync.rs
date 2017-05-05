
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::env;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::time;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::thread;
use std::borrow::Cow;


// TODO (?): use rust temp dir?
const FOLDER_NAME: &'static str = "tmp_contents";
const MANIFEST_TITLE: &'static str = ".4220_file_list.txt";
const BUFFER_SIZE: usize = 2048;

fn server(mut stream: TcpStream) -> io::Result<()> {
    let mut b = [b' '; BUFFER_SIZE];
    stream.read(&mut b)?;
    let s = String::from_utf8_lossy(&b);
    let mut tokens = s.splitn(3, " ");
    let command = tokens.nth(0);
    let file = tokens.nth(0).map(|b| Path::new(b.trim()));
    let body = tokens.nth(0);
    //println!("command: `{:?}`", command);
    //let fn_error_msg = String::from("ERROR: expected a filename");
    let response: Cow<str> = match command {
        Some("contents") => Cow::Owned(contents()?),
        Some("query") => match file {
            Some(filename) => Cow::Owned(query(filename)?.to_string()),
            None => Cow::Borrowed("ERROR: try `query <filename>`")
        },
        Some("get") => match file {
            Some(filename) => Cow::Owned(get(filename)?),
            None => Cow::Borrowed("ERROR: try `get <filename>`")
        },
        //Some("get_") => file.map_or_else(|f| get(f)?, || fn_error_msg),
        Some("put") => match (file,body) {
            (Some(f),Some(b)) => {
                put(f, b)?; 
                Cow::Borrowed("uh thanks (TODO)")
            },
            (Some(_),None) => Cow::Borrowed("ERROR: missing `body`"),
            (None,Some(_)) => Cow::Borrowed("ERROR: missing `filename`"),
            _ => Cow::Borrowed("ERROR: try `put <filename> <body>`"),
        },
        _ => Cow::Borrowed("ERROR: try command contents|query|get|put"),
    };
    //println!("Responding `{}`", response);
    stream.write_all(response.as_bytes())?;

    println!("foo");
    let mut b = [b' '; BUFFER_SIZE];
    stream.read(&mut b)?;
    //let mut t = String::new();
    //stream.read_to_string(&mut t)?;
    println!("AFTERWARDS got `{}`", b.len());

    Ok(())
}

fn build_file_path(p: &Path) -> io::Result<PathBuf> {
    let mut here = env::current_dir()?;
    here.push(p);
    Ok(here)
}

fn contents() -> io::Result<String> {
    get(Path::new(MANIFEST_TITLE))
}

fn query(filename: &Path) -> io::Result<u64> {
    // returns the seconds between 1970-01-01 00:00:00 and last modification
    let path = build_file_path(filename)?;
    let f = File::open(path)?;
    let md = f.metadata()?;
    let last_modified = md.modified()?;
    let age = last_modified.duration_since(time::UNIX_EPOCH).unwrap();
    Ok(age.as_secs())
}

fn get(filename: &Path) -> io::Result<String> {
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

/*
 * TODO: this will be used once we start update .420_whatever.txt
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
*/

fn parse_contents(contents: &str) -> Option<HashMap<String,u64>> {
    let mut v = HashMap::new();
    for line in contents.trim().split('\n') {
        let mut parts = line.splitn(2, "    ");
        let h = parts.nth(0);
        let f = parts.nth(0);
        if let (Some(hash),Some(name)) = (h, f) {
            let h: u64 = match hash.parse() {
                Ok(h) => h,
                Err(_) => return None,
            };
            v.insert(name.to_string(), h);
        } else {
            return None;
        }
    }
    Some(v)
}

fn client(mut stream: TcpStream) -> io::Result<()> {
    // compare client's folder with server's
    // if there are any files differet/missing, remedy that
    let mut s = String::new();
    stream.write(b"contents")?;
    stream.read_to_string(&mut s)?;
    let server_contents = parse_contents(&s)
        .ok_or(io::Error::new(io::ErrorKind::Other, "bad server contents"))?;
    let local_contents = get(Path::new(MANIFEST_TITLE))?;
    let client_contents = parse_contents(&local_contents)
        .ok_or(io::Error::new(io::ErrorKind::Other, "bad client contents"))?;
    println!("Server contents: {:?}", server_contents);
    println!("Client contents: {:?}", client_contents);
    for (name,cli_hash) in &client_contents {
        let path = Path::new(name);
        match server_contents.get(name) {
            Some(srv_hash) if srv_hash != cli_hash => {
                // server has a different copy of this file
                println!("query");
                let cmd = format!("query {}", name);
                stream.write(cmd.as_bytes())?;
                s.clear();
                stream.read_to_string(&mut s)?;
                // compare timestamps
                let srv_ts: u64 = s.parse().unwrap();
                let cli_ts: u64 = query(path)?;
                println!("Server touched {} at {} and client at {}", name, srv_ts, cli_ts);
                if cli_ts > srv_ts {
                    // client's version is newer
                    let text = get(path)?;
                    let cmd = format!("put {} {}", name, text);
                    stream.write(cmd.as_bytes())?;
                }
            },
            Some(_) => {
                println!("File {} has the same hashes", name);
            },
            None => {
                println!("Server didn't have {} at all", name);
                // server doesn't have this file at all
                let text = get(path)?;
                let cmd = format!("put {} {}", name, text);
                stream.write(cmd.as_bytes())?;
            },
        }
    }

    Ok(())
}

fn main() {
    let mode = env::args().nth(1).expect("USAGE: ./sync client|server");

    if mode == "server" {
        // switch directory
        let mut new_dir = env::current_dir().unwrap();
        new_dir.push(FOLDER_NAME);
        env::set_current_dir(new_dir).unwrap();
        // spawn listener
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        for stream in listener.incoming() {
            //server(stream.expect("Found invalid stream")).unwrap();
            thread::spawn(|| {
                server(stream.expect("Found invalid stream")).unwrap()
            });
        }
    } else if mode == "client" {
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        client(stream).unwrap();
    } else {
        panic!("Invalid mode string; it must be `client` or `server`");
    }
}
