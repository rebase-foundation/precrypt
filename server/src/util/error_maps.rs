use actix_web::HttpResponse;
use std::env::VarError;

pub fn var_error_map(e: VarError) -> HttpResponse {
   match e {
      VarError::NotPresent => HttpResponse::InternalServerError().body("Missing secret environment variable"),
       _ => HttpResponse::InternalServerError().finish(),
   }
}