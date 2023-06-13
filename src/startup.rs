use super::routes::{health_check, test_graph};

use axum::body::Body;
use axum::http::Request;
use axum::{routing::{get,post}, Router};
use std::future::Future;
use std::net::TcpListener;

use tower_http::trace::TraceLayer;
use tower_request_id::{RequestId, RequestIdLayer};

use crate::{
    configuration::Settings,
    routes::{generate_random_chart, selector_demo_path},
};
use axum_sessions::{
    async_session::MemoryStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer,
};
use rand::Rng;
use tower_http::services::ServeDir;

// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.

pub fn run(app: Application) -> impl Future<Output = Result<(), hyper::Error>> {
    let listener = app.listener;
    let fs = ServeDir::new("templates").append_index_html_on_directories(true);
    let store = MemoryStore::new();
    let secret = rand::thread_rng().gen::<[u8; 128]>();
    let session_layer = SessionLayer::new(store, &secret).with_secure(false);

    let app = Router::new()
        .fallback_service(fs)
        .route("/health_check", get(health_check))
        .route("/test_graph", get(test_graph))
        .route("/htmx_demo/selector/:name", get(selector_demo_path))
        .route("/htmx_demo/graph_data", get(generate_random_chart))
        .layer(
            // Let's create a tracing span for each request
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                // We get the request id from the extensions
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown".into());
                // And then we put it along with other information into the `request` span
                tracing::info!("Wassup");
                tracing::info_span!(
                    "request",
                    id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        // This layer creates a new id for each request and puts it into the request extensions.
        // Note that it should be added after the Trace layer.
        .layer(RequestIdLayer)
        .layer(session_layer)
        .layer(axum::Extension(reqwest::Client::new()));

    axum::Server::from_tcp(listener)
        .expect("Spawning server from listener failed")
        .serve(app.into_make_service())
}

// A new type to hold the newly built server and its port
pub struct Application {
    port: u16,
    listener: TcpListener,
}
impl Application {
    // We have converted the `build` function into a constructor for
    // `Application`.
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        tracing::info!("Spawning server on address {}", address);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        // We "save" the bound port in one of `Application`'s fields
        Ok(Self { listener, port })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        run(self).await
    }
}
