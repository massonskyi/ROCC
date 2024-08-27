use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;

use std::{error::Error, io, sync::Arc};

mod middleware;
mod core;
mod logger;
mod tools;
mod functions;

use tools::create_logger_file;
use logger::logger::AsyncLogger;
use core::database::db_core;
use functions::custom_errors;



use middleware::
{
    auth::
    {
        claims::validate, 
        handlers, 
        manager::UserManager
    },
    api::utopia_swagger
};

async fn render_start_page() -> &'static str {
    "Hello on start page!\ngo to \\tracks or \\users or \\playlists"
}

async fn index() -> impl Responder {
    
    HttpResponse::Ok().body(render_start_page().await)
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let file_path = match create_logger_file() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error creating log file path: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string())); // Convert to io::Error
        }
    };

    let logger = AsyncLogger::new(&file_path);
    match logger.init().await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error initializing logger: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string())); // Convert to io::Error
        }
    };


    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = 8050;

    let db_url = match db_core::establish_connection(){
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error creating db connection: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string())); // Convert to io::Error
        }
    };
    let bearer_middleware = HttpAuthentication::bearer(validate);
    let user_manager = Arc::new(tokio::sync::RwLock::new(
        UserManager::new(&db_url).await.expect("Failed to create UserManager")
    ));

    logger.b_info(&format!("Server started at http://{}:{}/", host, port)).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_manager.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .configure(handlers::init)
            .route("/", web::get().to(index))
            .route("/api-doc/openapi.json", web::get().to(utopia_swagger::get_openapi))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}