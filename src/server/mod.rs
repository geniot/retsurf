use std::{
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
    thread,
};


pub(super) fn config_port() -> u16 {
    if let Ok(port) = std::env::var("RETSURF_PORT") {
        return port
            .parse()
            .expect("Couldn't parse the RETSURF_PORT environment variable.");
    }
    8123
}

pub(super) fn config_dir() -> String {
    if let Ok(dir) = std::env::var("RETSURF_WEB_DIR") {
        return dir;
    }
    ".".to_string()
}

pub(crate) fn start_web_server() {
    thread::spawn(move || {
        let port = config_port();
        let dir = config_dir();

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port));

        println!("Static server is running on port {}", port);

        for con in listener.unwrap().incoming() {
            handle_stream(con.unwrap(), dir.to_string());
        }
    });
}

fn handle_stream(mut stream: TcpStream, directory: String) {
    thread::spawn(move || {
        // READING STREAM
        let stream_request_string = read_stream(&mut stream);
        let http_request = stream_request_string.split("\r\n").collect::<Vec<&str>>();

        let request_url = http_request[0].split(" ").collect::<Vec<&str>>()[1];

        // WRITING ANSWER
        let mut http_body: Vec<u8> = Vec::new();
        let file_path = || {
            if request_url == "/" {
                return format!("{}/{}", directory, "index.html");
            }
            format!("{}{}", directory, request_url)
        };

        file_to_http_body(file_path(), &mut http_body);

        stream.write(&http_body).expect("lol");
        stream.flush().unwrap();
    });
}

fn read_stream(stream: &mut TcpStream) -> String {
    let mut res_buf = vec![];
    loop {
        let mut buf = vec![0; 1000];
        stream.read(&mut buf).unwrap();
        res_buf.extend(buf.iter());

        let stringed = String::from_utf8_lossy(&res_buf);
        if stringed.contains("\r\n\r\n") {
            // END OF HEADERS
            break;
        }
    }

    let res_string = String::from_utf8_lossy(&res_buf).into_owned();

    res_string
        .split("\r\n\r\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()[0]
        .clone()
}

fn file_to_http_body(file_url: String, http_body: &mut Vec<u8>) {
    let mut file_content = Vec::new();
    let file = File::open(file_url.clone()).ok();

    if file.is_none() {
        http_body.extend("HTTP/1.1 404 NOT FOUND\r\n\r\n".as_bytes());
        return;
    }

    let file_mime_type = {
        let name = file_url.split("/").last().unwrap().to_string();
        let mime = mime_guess::from_path(name).first_or_octet_stream();

        mime.to_string()
    };

    file.unwrap()
        .read_to_end(&mut file_content)
        .expect("file err");

    let headers = [
        "HTTP/1.1 200 OK".to_string(),
        format!("Content-type: {}", file_mime_type),
        format!("Content-length: {}", file_content.len()),
        "\r\n".to_string(),
    ];

    http_body.extend(headers.join("\r\n").to_string().into_bytes());
    http_body.extend(file_content);
}
