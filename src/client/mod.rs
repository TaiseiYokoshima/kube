mod config;
mod client;

mod targets;
mod errors;
mod replicasets;
mod nodes;
mod pods;

mod get_replicasets;
mod get_nodes;
mod get_pods;

mod validate_targets;



mod watchers;

pub use client::{KubeClient, Base};
pub use targets::{Target, TargetInput};
pub use errors::{ClientError, TargetError, ErrorKind};
pub use replicasets::ReplicaSets;
pub use nodes::Nodes;
pub use pods::{Pods, PodSet, NamespaceMap};
