mod deployment;
mod replicaset;
mod pod;


pub use deployment::get_deployment_uuid;
pub use replicaset::{get_replicaset, ReplicaSetTarget};
pub use pod::get_pods_uids;
