use futures::{future, Future};
use tokio_core::reactor::Core;
use tokio_proto::TcpClient;
use tokio_service::Service;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::{io, str};

use std::fs;
use md5;

pub mod codec;
use self::codec::LineProto;


/*
 * Client should:
 *  Request and read 'contents' 
 *  For each content, compare w/ its local copy:
 *      If the md5 
 */

pub fn client(addr: SocketAddr) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // uhh this is where the whole protocol would go
	let client = TcpClient::new(LineProto);
    let stream = client.connect(&addr, &handle)
        .and_then(|client| client.call("contents".to_string())
            .and_then(|c| {
                println!("{}", c);
                //let srv_contents = parse_contents(&c).expect("bad server contents");
                //let cli_contents = get_local_contents().unwrap();
                //for (name, cli_hash) in &cli_contents {

                //}
                future::ok(())
            }))
        ;
        core.run(stream).unwrap();
}

fn get_local_contents() -> io::Result<HashMap<String,String>> {
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
            let hash = md5::compute(name.to_str().unwrap().as_bytes());
            contents.insert(name_str, format!("{:x}", hash));
        }
    }
    Ok(contents)
}

fn parse_contents(contents: &str) -> Option<HashMap<String,String>> {
    let mut v = HashMap::new();
    for line in contents.trim().split('\n') {
        let mut parts = line.splitn(2, "    ");
        let h = parts.nth(0);
        let f = parts.nth(0);
        if let (Some(hash),Some(name)) = (h, f) {
            v.insert(name.to_string(), hash.to_string());
        } else {
            return None;
        }
    }
    Some(v)
}


