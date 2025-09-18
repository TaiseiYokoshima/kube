mod input;
mod client;
mod config;


// mod get_replicasets;
// mod get_deployments;
// mod validate_namespace;


// mod get_nodes;
mod validate_target;

pub use client::KubeClient;
pub use input::{Target, TargetInput, TargetError, Kind};



#[derive(Debug)]
pub enum ClientError {
   Api(reqwest::Error),
   Serde(serde_json::Error),
   Json(&'static str),
   TargetValidationError(Vec<TargetError>),
}

impl From<reqwest::Error> for ClientError {
   fn from(value: reqwest::Error) -> Self {
       ClientError::Api(value)
   }
}

impl From<serde_json::Error> for ClientError {
   fn from(value: serde_json::Error) -> Self {
      ClientError::Serde(value)
   }
}
