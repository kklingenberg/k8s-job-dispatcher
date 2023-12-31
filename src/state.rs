//! Defines the common internal application state.

use crate::jq;

use k8s_openapi::api::batch::v1::Job;
use kube::{api::Api, Client};
use std::sync::Arc;

/// Internal application state
#[derive(Clone)]
pub struct AppState {
    pub filter: Arc<jq::Filter>,
    pub k8s_client: Arc<Client>,
    pub k8s_jobs: Arc<Api<Job>>,
}

impl AppState {
    /// Create a new instance of AppState
    pub fn new(filter: jq::Filter, k8s_client: Client, k8s_jobs: Api<Job>) -> Self {
        Self {
            filter: Arc::new(filter),
            k8s_client: Arc::new(k8s_client),
            k8s_jobs: Arc::new(k8s_jobs),
        }
    }
}
