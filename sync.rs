
use std::{env, time, thread};
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::collections::{HashMap, hash_map};
//use std::borrow::Cow;

// TODO (?): use rust temp dir?
const FOLDER_NAME: &'static str = "tmp_contents";
const MANIFEST_TITLE: &'static str = ".4220_file_list.txt";
const BUFFER_SIZE: usize = 2048;
const USAGE: &'static str = "USAGE: ./syncr client|server <port>";

fn server(mut stream: TcpStream) -> io::Result<()> {
    let mut b: [u8; BUFFER_SIZE];
    loop {
        b = [0; BUFFER_SIZE];
        let len = stream.read(&mut b)?;
        if len == 0 {
            return Ok(())
        }
        let mut tokens = b[..len].splitn(3, |i| *i==b' ');
        let command = tokens.nth(0);
        let file = tokens.nth(0);//.map(|f| f.trim());
        let path_s = file.map(|b| String::from_utf8_lossy(b));
        let path = path_s.map(|s| PathBuf::from(s.into_owned()));
        let body = tokens.nth(0);
        let response: Vec<u8> = match command {
            Some(b"contents") => contents()?,
            Some(b"query") => match path {
                Some(filename) => query(&filename)?.to_string().as_bytes().to_vec(),
                None => "ERROR: try `query <filename>`".as_bytes().to_vec()
            },
            Some(b"get") => match path {
                Some(filename) => get(&filename)?,
                None => "ERROR: try `get <filename>`".as_bytes().to_vec()
            },
            Some(b"put") => match (file,body) {
                (Some(f),Some(b)) => {
                    let mut srv_contents = get_local_contents()?;
                    let new_hash = hash_bytes(b);
                    let filename = String::from_utf8_lossy(f).into_owned();
                    put(Path::new(&filename), b)?; 
                    srv_contents.insert(filename, new_hash); 
                    put_local_contents(srv_contents)?;
                    continue;
                },
                (Some(_),None) => "ERROR: missing `body`".as_bytes(),
                (None,Some(_)) => "ERROR: missing `filename`".as_bytes(),
                _ => "ERROR: try `put <filename> <body>`".as_bytes(),
            }.to_vec(),
            _ => "ERROR: try command contents|query|get|put".as_bytes().to_vec(),
        };
        stream.write(&response)?;
    }
}

fn build_file_path(p: &Path) -> io::Result<PathBuf> {
    let mut here = env::current_dir()?;
    here.push(p);
    Ok(here)
}

fn contents() -> io::Result<Vec<u8>> {
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

fn get(filename: &Path) -> io::Result<Vec<u8>> {
    let path = build_file_path(filename)?;
    let mut f = File::open(path)?;
    //let mut s = String::new();
    let mut v = Vec::new();
    //f.read_to_string(&mut s)?;
    f.read_to_end(&mut v)?;
    Ok(v)
}

fn put(filename: &Path, contents: &[u8]) -> io::Result<()> {
    let path = build_file_path(filename)?;
    let mut f = File::create(path)?;
    //f.write_all(contents.as_bytes())
    f.write_all(contents)
}

 // TODO: this will be used once we start update .420_whatever.txt
fn hash(filename: &Path) -> io::Result<u64> {
    // NOTE: this uses the hash algorithm used by rust's hashmap
    //  because it's the only hash algorithm in the Rust std lib
    // it would be trivial to use the md5 crate, but this requires no crates 
    // todo fix? idk
    let text = get(filename)?;
    let mut s = hash_map::DefaultHasher::new();
    text.hash(&mut s);
    Ok(s.finish())
}

fn hash_bytes(text: &[u8]) -> u64 {
    let mut s = hash_map::DefaultHasher::new();
    text.hash(&mut s);
    s.finish()
}
    

fn get_local_contents() -> io::Result<HashMap<String,u64>> {
    let rd = fs::read_dir(".")?;
    let mut contents = HashMap::new();
    for file in rd {
        let path = file?;
        if path.path().is_file() {
            let name = path.file_name();
            let name_str = name.to_string_lossy().into_owned();
            if name_str.starts_with('.') {
                continue;
            }
            let hash = hash(Path::new(&name))?;
            contents.insert(name_str, hash);
        }
    }
    Ok(contents)
}

fn put_local_contents(c: HashMap<String,u64>) -> io::Result<()> {
    let mut s = String::new();
    for (name,hash) in c {
        let f = format!("{h:>0w$}    {file}\n", h=hash, w=21, file=name);
        s.push_str(&f);
    }
    let mut f = File::create(Path::new(MANIFEST_TITLE))?;
    f.write_all(s.as_bytes())?;
    Ok(())
}

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
    stream.write(b"contents")?;

    let mut b = [0u8; BUFFER_SIZE];
    let len = stream.read(&mut b)?;
    let s = String::from_utf8_lossy(&b[..len]);

    let client_contents = get_local_contents()?;
    let server_contents = parse_contents(&s)
        .ok_or(io::Error::new(io::ErrorKind::Other, "bad server contents"))?;

    for (name,cli_hash) in &client_contents {
        let mut b = [0u8; BUFFER_SIZE];
        let path = Path::new(name);
        match server_contents.get(name) {
            Some(srv_hash) if srv_hash != cli_hash => {
                // server has a different copy of this file
                let cmd = format!("query {}", name);
                stream.write(cmd.as_bytes())?;

                let len = stream.read(&mut b)?;
                let s = String::from_utf8_lossy(&b[..len]);
                
                // compare timestamps
                let srv_ts: u64 = s.parse().unwrap();
                let cli_ts: u64 = query(path)?;
                println!("Server touched {} at {} and client at {}", name, srv_ts, cli_ts);
                if cli_ts > srv_ts {
                    // client's version is newer
                    let mut text = get(path)?;
                    let mut prefix = b"put ".to_vec();
                    let mut name_b = name.as_bytes().to_vec();
                    let mut cmd = vec![];
                    cmd.append(&mut prefix);
                    cmd.append(&mut name_b);
                    cmd.push(b' ');
                    cmd.append(&mut text);
                    stream.write(&cmd)?;
                } else {
                    // client's version is older
                    let cmd = format!("get {}", name);
                    stream.write(cmd.as_bytes())?;
                    let mut b = [0u8; BUFFER_SIZE];
                    let len = stream.read(&mut b)?;
                    put(Path::new(name), &b[..len])?;
                }
            },
            Some(_) => {
                println!("File {} has the same hashes", name);
            },
            None => {
                println!("Server didn't have {} at all", name);
                // server doesn't have this file at all
                let mut text = get(path)?;
                let mut prefix = b"put ".to_vec();
                let mut name_b = name.as_bytes().to_vec();
                let mut cmd = vec![];
                cmd.append(&mut prefix);
                cmd.append(&mut name_b);
                cmd.push(b' ');
                cmd.append(&mut text);
                stream.write(&cmd)?;
            },
        }
    }
    Ok(())
}

fn main() {
    let mut args = env::args();
    let mode = args.nth(1).expect(USAGE);
    let port: u16 = args.nth(0).expect(USAGE).parse().expect(USAGE);

    if mode == "server" {
        // switch directory
        let mut new_dir = env::current_dir().unwrap();
        new_dir.push(FOLDER_NAME);
        env::set_current_dir(new_dir).unwrap();
        // spawn listener
        let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        for stream in listener.incoming() {
            thread::spawn(|| {
                server(stream.expect("Found invalid stream")).unwrap()
            });
        }
    } else if mode == "client" {
        let stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client(stream).unwrap();
    } else {
        panic!("Invalid mode string; it must be `client` or `server`");
    }
}


