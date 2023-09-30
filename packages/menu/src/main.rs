use actix_cors::Cors;
use actix_web::{
    http, middleware,
    web::{self},
    Result,
};
use lambda_runtime::Error;
use lambda_web::actix_web::{self, App, HttpServer};
use lambda_web::{is_running_on_lambda, run_actix_on_lambda};
use moosicbox_core::app::AppState;
use moosicbox_menu::api;
use std::{env, time::Duration};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    let service_port = 8000;

    let factory = move || {
        let proxy_client = awc::Client::builder()
            .timeout(Duration::from_secs(120))
            .finish();

        let proxy_url = env::var("PROXY_HOST")
            .unwrap_or_else(|_e| panic!("Missing PROXY_HOST environment variable"));

        let image_client = awc::Client::builder()
            .timeout(Duration::from_secs(120))
            .finish();

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(AppState {
                service_port,
                proxy_url,
                proxy_client,
                image_client,
                db: None,
            }))
            .service(api::get_artists_endpoint)
            .service(api::get_albums_endpoint)
            .service(api::get_album_tracks_endpoint)
    };

    if is_running_on_lambda() {
        run_actix_on_lambda(factory).await?;
    } else {
        HttpServer::new(factory)
            .bind(format!("0.0.0.0:{service_port}"))?
            .run()
            .await?;
    }
    Ok(())
}
