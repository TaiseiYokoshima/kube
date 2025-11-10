mod client;
mod config;

mod daemon_set;
mod error;

mod parse_json_pod;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::Status;

pub use client::{Base, KubeClient};
pub use daemon_set::{CAdvisorDaemonSetMetadata, CAdvisorPods, get_daemon_set_pods, Watcher, WatcherError, DaemonSetEvent, Pod};
pub use error::{APIError, JsonQuery, response_into_error, errors};





pub type Uid = Box<str>;
pub type ResourceVersion = Box<str>;
pub type KubeErrorStatus = Status;
