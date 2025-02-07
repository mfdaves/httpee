/*
** todo: implement default trait on ServerOptions, and rewrite run_server,
** error handling, add middleware and cache handler, drop server command. 
*/

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Request, Response, Server, StatusCode};
use mime_guess;

pub struct Router {
    routes: HashMap<(String, String), Box<dyn Fn(Request)>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }
    pub fn add_route(&mut self, method: &str, endpoint: &str, function: Box<dyn Fn(Request)>) {
        self.routes.insert((method.to_string(), endpoint.to_string()), function);
    }
    pub fn request_handler(&self, req: Request) {
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

pub struct ServerOptions<'a> {
    public_folders: Option<Vec<&'a PathBuf>>,
	port: u16,
	//..
}

impl<'a> ServerOptions<'a> {
    pub fn new(public_folders: Option<Vec<&'a PathBuf>>, port: u16) -> Self {
        ServerOptions { public_folders, port}
    }
	
	pub fn get_port(&self)-> u16{
		self.port
	}
	
	pub fn get_public_folders(&self)->Option<Vec<&'a PathBuf>>{
		self.public_folders.clone()
	}
}

impl Default for ServerOptions<'_> {
	fn default () -> Self {
		ServerOptions{
			public_folders: None, 
			port: 8000
		}
	}
}




pub struct ServerUtilities;

impl ServerUtilities {
    pub fn serve_error(request: Request) {
        eprintln!("WARNING:: ENDPOINT NOT FOUND!");
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

    pub fn serve_static(public: &PathBuf, router: &mut Router) {
		if public.is_dir() {
			for entry in public.read_dir().unwrap() {
				let entry = entry.unwrap();
				let path = entry.path();
				if path.is_dir() {
					ServerUtilities::serve_static(&path, router);
				} else {
					// let relative_path = path.strip_prefix(public).unwrap();
					let endpoint = format!("/{}", path.to_str().unwrap().replace("\\", "/"));
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
	
	
	pub fn public_folders_handler(public_folders: Vec<&PathBuf>, router: &mut Router){
		for public in public_folders{
			ServerUtilities::serve_static(public,router);
		}
	}
	
	
	
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
	
	//rewrite based on ServerOptions
    pub fn run_server(server_options: ServerOptions){
		let port = server_options.get_port();
		let public_folders = server_options.get_public_folders();
		let address = format!("0.0.0.0:{}", port);
		println!("{}", address);
        let server = Server::http(address).unwrap();
        println!("HTTP server listening on port {}!", port);
        let mut router = Router::new();
		if let Some(folders) = public_folders {
			ServerUtilities::public_folders_handler(folders, &mut router);
		} else {
			println!("No public folders provided...");
		} 
        for request in server.incoming_requests() {
            router.request_handler(request);
        }
    }
}


