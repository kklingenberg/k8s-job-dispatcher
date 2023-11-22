//! Implements the creation and retrieval of K8s jobs.

use crate::jq;
use crate::state;

use actix_web::{error, get, post, web, HttpResponse, Responder, Result};
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use kube::{core::params::PostParams, Error};
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, info};

/// A reduced representation of a K8s job.
#[derive(Serialize)]
struct JobSummary {
    id: Option<String>,
    status: Option<JobStatus>,
}

/// Create a K8s job by converting the request body to a job manifest.
#[post("/job")]
async fn create_job(
    body: web::Json<Value>,
    state: web::Data<state::AppState>,
) -> Result<impl Responder> {
    debug!("Job creation request: {:?}", body);
    let ir = jq::first_result(&state.request_to_ir, body.into_inner())
        .ok_or_else(|| error::ErrorBadRequest("IR filter didn't produce results"))?
        .map_err(|e| error::ErrorBadRequest(format!("IR filter failed: {:?}", e)))?;
    debug!("Job IR: {:?}", ir);
    let raw_manifest = jq::first_result(&state.ir_to_manifest, ir)
        .ok_or_else(|| error::ErrorBadRequest("Manifest filter didn't produce results"))?
        .map_err(|e| error::ErrorBadRequest(format!("Manifest filter failed: {:?}", e)))?;
    debug!("Job raw manifest: {:?}", raw_manifest);
    let manifest: Job = serde_json::from_value(raw_manifest)
        .map_err(|e| error::ErrorBadRequest(format!("Generated manifest is invalid: {:?}", e)))?;
    debug!("Job manifest: {:?}", manifest);
    let job_opt = state
        .k8s_jobs
        .create(&PostParams::default(), &manifest)
        .await
        .map_or_else(
            |e| match e {
                Error::Api(response) if response.code == 409 => Ok(None),
                _ => Err(error::ErrorBadRequest(format!(
                    "K8s server rejected job manifest: {:?}",
                    e
                ))),
            },
            |job| Ok(Some(job)),
        )?;
    if let Some(job) = job_opt {
        info!(
            "Created job with ID {:?}",
            job.metadata
                .name
                .clone()
                .unwrap_or_else(|| String::from("<unknown>"))
        );
        Ok(HttpResponse::Created().json(JobSummary {
            id: job.metadata.name,
            status: job.status,
        }))
    } else {
        info!(
            "Pre-existing job with ID {:?}",
            manifest
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| String::from("<unknown>"))
        );
        Ok(HttpResponse::Ok().json(JobSummary {
            id: manifest.metadata.name,
            status: manifest.status,
        }))
    }
}

/// Fetch a K8s job by its ID.
#[get("/job/{id}")]
async fn get_job(
    id: web::Path<String>,
    state: web::Data<state::AppState>,
) -> Result<impl Responder> {
    let job = state
        .k8s_jobs
        .get_opt(&id)
        .await
        .map_err(error::ErrorBadGateway)?
        .ok_or_else(|| error::ErrorNotFound("The specified job doesn't exist"))?;
    info!(
        "Fetched job with ID {:?}",
        job.metadata
            .name
            .clone()
            .unwrap_or_else(|| String::from("<unknown>"))
    );
    Ok(web::Json(JobSummary {
        id: job.metadata.name,
        status: job.status,
    }))
}
