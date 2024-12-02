use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Request, Response, Server, StatusCode};
use mime_guess;

struct Router {
    routes: HashMap<(String, String), Box<dyn Fn(Request)>>,
}

impl Router {
    fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    fn add_route(&mut self, method: &str, endpoint: &str, function: Box<dyn Fn(Request)>) {
        self.routes.insert((method.to_string(), endpoint.to_string()), function);
    }

    fn request_handler(&self, req: Request) {
        let endpoint = req.url().to_string();
        let method = req.method().as_str().to_string();
        println!(
            "INFO:: A new incoming request: endpoint={:?}, method={:?}",
            endpoint, method
        );
        if let Some(handler) = self.routes.get(&(method, endpoint)) {
            handler(req);
        } else {
            ServerUtilities::serve_error(req);  
        }
    }
}

struct ServerOptions<'a> {
    public_folders: Option<Vec<&'a PathBuf>>,
}

impl<'a> ServerOptions<'a> {
    fn new(publics: Option<Vec<&'a PathBuf>>) -> Self {
        ServerOptions { public_folders: publics }
    }
}

struct ServerUtilities;

impl ServerUtilities {
    pub fn serve_error(request: Request) {
        println!("WARNING:: ENDPOINT NOT FOUND!");
        let error_path = Path::new("public/404.html");
        let file = match File::open(error_path) {
            Ok(file) => file,
            Err(_) => {
                let response = Response::from_string("Error 404: Not Found! Try with /hello")
                    .with_status_code(StatusCode(404));
                request.respond(response).unwrap();
                return;
            }
        };
        let response = Response::from_file(file)
            .with_header(Header::from_bytes("Content-Type", "text/html; charset=UTF-8").unwrap())
            .with_status_code(StatusCode(404));
        request.respond(response).unwrap();
    }

    pub fn serve_static(public_folder: &PathBuf, router: &mut Router) {
        if public_folder.is_dir() {
            for entry in public_folder.read_dir().unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    ServerUtilities::serve_static(&path, router);
                } else {
                    let relative_path = path.strip_prefix(public_folder).unwrap();
                    let endpoint = format!("/{}", relative_path.to_str().unwrap().replace("\\", "/"));

                    let action = move |req: Request| {
                        ServerUtilities::serve_file(req, path.clone());
                    };

                    router.add_route("GET", &endpoint, Box::new(action));

                    println!("Serving file at endpoint: {}", endpoint);
                }
            }
        } else {
            panic!("This folder doesn't exist! Check the provided path!...");
        }
    }

    // Serve un file statico
    pub fn serve_file(request: Request, file_path: PathBuf) {
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(_) => {
                let response = Response::from_string("Error 404: File Not Found")
                    .with_status_code(StatusCode(404));
                request.respond(response).unwrap();
                return;
            }
        };

        let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
        let response = Response::from_file(file)
            .with_header(Header::from_bytes("Content-Type", mime_type.to_string()).unwrap());
        request.respond(response).unwrap();
    }

    pub fn run_server(server_options: Option<ServerOptions>) {
        let server = Server::http("0.0.0.0:8000").unwrap();
        println!("HTTP server listening on port 8000!");
        let mut router = Router::new();
        let public_folder_path = &PathBuf::from("public");
        
        match server_options {
            Some(server_options) => {
                let public_folders = server_options.public_folders;
                println!("{:?}", public_folders);
                if let Some(publics) = public_folders {
                    for single_public in publics {
                        ServerUtilities::serve_static(single_public, &mut router);  // Usa la funzione di ServerUtilities
                    }
                }
            },
            None => {}
        }
        
        router.add_route(
            "GET",
            "/hello",
            Box::new(|req: Request| {
                let response = Response::from_string("Hello, World!")
                    .with_header(Header::from_bytes("Content-Type", "text/plain; charset=UTF-8").unwrap());
                req.respond(response).unwrap();
            }),
        );

        for request in server.incoming_requests() {
            router.request_handler(request);
        }
    }
}



//******** Modules allow you to have different place to store your services by topic and utilities
//******** serve_static allow you to serve all the static file you put in a specific folder inside the project
//******** you could customize your error handling serving an error pages or just a msg with status code 404
//******** next I'll add middleware like logging middleware, parameters validator etc etc.. so if users doesn't provide you the correct prm you'll send a error msg
//******** -> and specific prm you dont 


