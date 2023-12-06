//! Implements the creation and retrieval of K8s jobs.

use crate::api_error::APIError;
use crate::jq;
use crate::state;

use actix_web::{get, routes, web, HttpResponse, Responder, Result};
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use kube::{core::params::PostParams, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info};

/// A reduced representation of a K8s job.
#[derive(Serialize)]
struct JobSummary {
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<JobStatus>,
}

/// A container for the create_job path information.
#[derive(Deserialize)]
struct PathInfo {
    path: Option<String>,
}

/// Create a K8s job by converting the request body to a job manifest.
#[routes]
#[post("/job")]
#[post("/job/{path:.*}")]
async fn create_job(
    path: web::Path<PathInfo>,
    body: web::Json<Value>,
    state: web::Data<state::AppState>,
) -> Result<impl Responder> {
    let path = format!("/job/{}", path.path.clone().unwrap_or_default());
    let path = path.strip_suffix('/').map(String::from).unwrap_or(path);
    debug!("Job creation request at {:?}: {:?}", path, body);
    let raw_manifest = jq::first_result(&state.filter, body.into_inner(), &path)
        .ok_or_else(|| APIError::bad_request("Filter didn't produce results"))?
        .map_err(|e| APIError::bad_request(format!("Filter failed: {:?}", e)))?;
    debug!("Job raw manifest: {:?}", raw_manifest);
    let manifest: Job = serde_json::from_value(raw_manifest)
        .map_err(|e| APIError::bad_request(format!("Generated manifest is invalid: {:?}", e)))?;
    debug!("Job manifest: {:?}", manifest);
    let job_opt = state
        .k8s_jobs
        .create(&PostParams::default(), &manifest)
        .await
        .map_or_else(
            |e| match e {
                Error::Api(response) if response.code == 409 => Ok(None),
                _ => Err(APIError::bad_request(format!(
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
            status: None,
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
            status: None,
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
        .map_err(APIError::bad_gateway)?
        .ok_or_else(|| APIError::not_found("The specified job doesn't exist"))?;
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
