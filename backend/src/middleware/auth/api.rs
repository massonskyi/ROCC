use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{claims::Claims, manager::UserManager, api_models::{LoginRequest, SignUpRequest}};  // Assuming UserManager is implemented elsewhere

#[utoipa::path(
    get,
    request_body = LoginRequest,
    path = "/sign_in",
    responses(
        (status = 200, description= "successful response", body=LoginRequest),
        (status = 401, description= "unauthorized response", body=String),
        (status = 403, description= "forbidden response", body=String),
        (status = 500, description= "internal server error", body=String),
    )
)]
pub async fn sign_in(
    user_manager: web::Data<Arc<RwLock<UserManager>>>, 
    req: web::Json<LoginRequest>
) -> impl Responder {
    // Obtain mutable access to UserManager
    let manager = user_manager.read().await;

    // Retrieve user by username (assuming UserManager has a method to find user by username)
    let user = match manager.get_user_by_username(&req.username).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().finish(), // User not found
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Verify password using bcrypt
    let password_match = match verify(&req.password, &user.hash_password) {
        Ok(matched) => matched,
        Err(_) => return HttpResponse::Unauthorized().finish(), // Error verifying password
    };

    if !password_match {
        return HttpResponse::Unauthorized().finish(); // Incorrect password
    }

    // Generate JWT token for the authenticated user
    let claims = Claims {
        sub: user.id,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize, // Token expires in 24 hours
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret"), // Replace with your actual secret key
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Return token as JSON response
    HttpResponse::Ok().json(token)
}

#[utoipa::path(
    post,
    path = "/sign_up",
    request_body = SignUpRequest,
    responses(
        (status = 200, description = "Successful response", body = String), // JWT token as a string
        (status = 401, description = "Unauthorized response", body = String),
        (status = 403, description = "Forbidden response", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn sign_up(
    // Login Request: web::Json<LoginRequest> -> impl Responder
    user_manager: web::Data<Arc<RwLock<UserManager>>>, 
    req: web::Json<SignUpRequest>
) -> impl Responder{

    let hashed_password = match hash(&req.password, 10) {
        Ok(hashed) => {
            println!("{}", hashed);
            hashed
        },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Clone necessary fields from SignUpRequest
    let username = req.username.clone();
    let email = req.email.clone();
    let role = req.role.clone();
    let avatar = req.avatar.clone();
    println!("{username} {email} {role}, {avatar} {0} {1} {2}", req.name.clone(),req.surname.clone(), req.age);
    // Obtain mutable access to UserManager
    let manager = user_manager.write().await;

    // Create the user in the database
    let user = match manager.create_user(
        req.name.clone(),     // Clone other fields as needed
        req.surname.clone(),
        req.age,
        username,
        email,
        hashed_password,
        role,
        avatar,
    ).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Generate JWT token for the newly created user
    let claims = Claims {
        sub: user.id,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize, // Token expires in 24 hours
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test"), // Replace with your actual secret key
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    println!("{token}");
    // Return token as JSON response
    HttpResponse::Ok().json(token)
}