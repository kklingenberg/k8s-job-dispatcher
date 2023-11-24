mod health_service;
mod jq;
mod k8s_service;
mod state;

use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use k8s_openapi::api::batch::v1::Job;
use kube::{api::Api, Client, Config};
use std::path::PathBuf;
use std::time::Duration;
use tracing::warn;

const DEFAULT_FILTER: &str = include_str!("default_filter.jq");

/// Job-dispatching interface acting as a thin wrapper over K8s API.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Filter converting requests to K8s manifests
    filter: Option<String>,

    /// Read filter from a file
    #[arg(short, long, env)]
    from_file: Option<PathBuf>,

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
    let filter_source = if let Some(filter_file) = cli.from_file {
        if cli.filter.is_some() {
            warn!("Filter given both as file and argument; argument will be ignored");
        }
        std::fs::read_to_string(filter_file)
    } else if let Some(filter_str) = cli.filter {
        Ok(filter_str)
    } else {
        warn!("No filter given; the default filter will be used");
        Ok(DEFAULT_FILTER.to_string())
    }?;
    let filter = jq::compile(&filter_source).expect("error compiling filter");
    let mut k8s_config = Config::infer()
        .await
        .expect("error detecting K8s configuration");
    k8s_config.connect_timeout = Some(Duration::from_secs(15));
    k8s_config.read_timeout = Some(Duration::from_secs(15));
    k8s_config.write_timeout = Some(Duration::from_secs(15));
    let k8s_client = Client::try_from(k8s_config).expect("error initializing K8s client");
    let k8s_jobs: Api<Job> = Api::default_namespaced(k8s_client.clone());
    let appstate = web::Data::new(state::AppState::new(filter, k8s_client, k8s_jobs));

    // Boot the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::trim())
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
