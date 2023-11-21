mod health_service;
mod jq;
mod k8s_service;
mod state;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use k8s_openapi::api::batch::v1::Job;
use kube::{api::Api, Client, Config};
use std::time::Duration;

const DEFAULT_REQUEST_TO_IR_FILTER: &str = include_str!("request_to_ir.jq");
const DEFAULT_IR_TO_MANIFEST_FILTER: &str = include_str!("ir_to_manifest.jq");

/// Job-dispatching interface acting as a thin wrapper over K8s API.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Filter converting requests to internal representation
    #[arg(long, env)]
    request_to_ir: Option<String>,

    /// Filter converting internal representation to a K8s manifest
    #[arg(long, env)]
    ir_to_manifest: Option<String>,

    /// TCP port to listen on
    #[arg(short, long, env, default_value_t = 8000)]
    port: u16,

    /// Log level
    #[arg(long, env, default_value_t = tracing::Level::INFO)]
    log_level: tracing::Level,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level)
        .with_target(false)
        .without_time()
        .init();

    // Initialize application state
    let request_to_ir = jq::compile(
        &cli.request_to_ir
            .unwrap_or_else(|| String::from(DEFAULT_REQUEST_TO_IR_FILTER)),
    )
    .expect("error compiling request-to-ir filter");
    let ir_to_manifest = jq::compile(
        &cli.ir_to_manifest
            .unwrap_or_else(|| String::from(DEFAULT_IR_TO_MANIFEST_FILTER)),
    )
    .expect("error compiling ir-to-manifest filter");
    let mut k8s_config = Config::infer()
        .await
        .expect("error detecting K8s configuration");
    k8s_config.connect_timeout = Some(Duration::from_secs(15));
    k8s_config.read_timeout = Some(Duration::from_secs(15));
    k8s_config.write_timeout = Some(Duration::from_secs(15));
    let k8s_client = Client::try_from(k8s_config).expect("error initializing K8s client");
    let k8s_jobs: Api<Job> = Api::default_namespaced(k8s_client.clone());
    let appstate = web::Data::new(state::AppState::new(
        request_to_ir,
        ir_to_manifest,
        k8s_client,
        k8s_jobs,
    ));

    // Boot the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(appstate.clone())
            .service(health_service::liveness_check)
            .service(health_service::readiness_check)
            .service(k8s_service::create_job)
            .service(k8s_service::get_job)
    })
    .bind(("0.0.0.0", cli.port))?
    .run()
    .await
}
