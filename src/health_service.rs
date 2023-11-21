//! Implements the liveness and readiness checks.

use crate::state;

use actix_web::{error, get, web, HttpResponse, Responder, Result};

/// Liveness check: if this function can execute, the process is
/// alive.
#[get("/health/live")]
async fn liveness_check() -> impl Responder {
    HttpResponse::NoContent().finish()
}

/// Readiness check: if the K8s API responds, the process is ready to
/// receive commands.
#[get("/health/ready")]
async fn readiness_check(state: web::Data<state::AppState>) -> Result<impl Responder> {
    state
        .k8s_client
        .apiserver_version()
        .await
        .map_err(error::ErrorServiceUnavailable)?;
    Ok(HttpResponse::NoContent().finish())
}
