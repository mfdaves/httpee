use std::fs::File;
use std::collections::HashMap;
use tiny_http::{Server, Request, Response, Header, StatusCode};
use std::path::Path;


struct Router {
	routes:HashMap<(&'static str,&'static str),fn(Request)>,
}

impl Router {
	fn new () -> Self {
		Router {
			routes:HashMap::new()
		}
	}
	
	fn add_route(&mut self, method:&'static str, endpoint: &'static str, function:fn(Request)){
		self.routes.insert((method,endpoint),function);
	}
	
	fn request_handler (&self,req:Request){
		println!("{:?}",req);
		let endpoint = req.url();
		let method = req.method().as_str();
		println!("INFO:: a new incoming request: endpoint={:?} and method={:?}", endpoint, method);
		if let Some(handler) = self.routes.get(&(method,endpoint)){
			handler(req);
		} else {
			serve_error(req);
		}
	}
}

fn main () {
	run_server();
}


fn hello_world (request:Request){
	let hello_world_path = Path::new("public/hello.html");
	let file = File::open(hello_world_path).unwrap();
	let response = Response::from_file(file)
					.with_header(Header::from_bytes("Content-Type", "text/html; charset=UTF-8").unwrap());
	request.respond(response).unwrap();
}




fn serve_error(request:Request){
	println!("WARNING:: ENDPOINT DOESN'T FOUND!");
	let error_path = Path::new("public/404.html");
	let file = match File::open(error_path){
		Ok(file)=>file, 
		Err(_)=> {
			let response = Response::from_string("Error 404 not found! Try with /hello")
							.with_status_code(StatusCode(404));
			request.respond(response).unwrap();
			return;
			}		
	};
	let response = Response::from_file(file)
					.with_header(Header::from_bytes("Content-Type","text/html; charset=UTF-8").unwrap())
					.with_status_code(StatusCode(404));
					
	request.respond(response).unwrap();
}




fn run_server(){
	let server = Server::http("0.0.0.0:8000").unwrap();
	println!("HTTP server listening on port 8000!");
	
	let mut router = Router::new();
	
	router.add_route("GET","/hello",hello_world/*maybe aggiungere come campo anche i parametri, i quali devono essere mandatory*/);
	for request in server.incoming_requests(){
		router.request_handler(request);
	}
}



// fn serve_static(public_folder:Path,router:&mut Router) {
	// if public_folder.is_dir(){
		// for entry in public_folder.read_dir()? {
			// let entry = entry?;
			// let path = entry.path();
			// if path.is_dir{
				// serve_static(path,&mut router);
			// }else{
				// router.add_route("GET",path.to_str,serve_file);
			// }
		// }
	// }else {
		// panic!("This folder doesn't exists! Check the provided path!...");
	// }
// }


// fn serve_file (request:Request,file_path:Path){
	// let hello_world_path = Path::new("public/hello.html");
	// let file = File::open(hello_world_path).unwrap();
	
	// let response = Response::from_file(file)
					// .with_header(Header::from_bytes("Content-Type", "text/html; charset=UTF-8").unwrap());
	// request.respond(response).unwrap();
// }
