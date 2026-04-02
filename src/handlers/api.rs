use actix_web::{web, HttpResponse};
use rusqlite::Connection;
use std::sync::Mutex;

use crate::models::settings::SiteSetting;

pub async fn get_settings(
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let conn = db.lock().unwrap();
    let settings = SiteSetting::get_all(&conn).unwrap_or_default();
    HttpResponse::Ok().json(settings)
}
