use std::collections::HashMap;

use k8s_openapi::{List, api::apps::v1::ReplicaSet};

use super::ClientError;
use super::ReplicaSets;
use super::Target;
use super::client::Base;

use super::replicasets::PodsCount;

fn is_owned(set: &ReplicaSet, targets: &Vec<Target>) -> Result<bool, ClientError> {
   for target in targets {
      let uid = target.deployment_uid.as_ref();

      let owners = set
         .metadata
         .owner_references
         .as_ref()
         .ok_or(ClientError::Json(
            "no owner references in replica set's metadata",
         ))?;

      let owned = owners.iter().find(|owner| owner.uid == uid).is_some();

      if owned {
         return Ok(true);
      }
   }

   Ok(false)
}

pub async fn get_replica_sets(
   client: &Base,
   targets: &Vec<Target>,
) -> Result<ReplicaSets, ClientError> {
   let endpoint = format!("/apis/apps/v1/replicasets");
   let response = client.get(endpoint).send().await;

   let response_body = response?.error_for_status()?.text().await?;

   let replicasets = serde_json::from_str::<List<ReplicaSet>>(&response_body)?;

   let mut hashes: HashMap<Box<str>, PodsCount> = HashMap::new();

   for set in replicasets.items.iter() {
      if !is_owned(set, targets)? {
         continue;
      };

      let hash = set
         .metadata
         .labels
         .as_ref()
         .ok_or(ClientError::Json("no labels in replica set"))?
         .get("pod-template-hash")
         .ok_or(ClientError::Json("no hash in replica set's labels"))?
         .clone()
         .into();

      let pods_count = set.status.as_ref().ok_or(ClientError::Json("no status in replica set"))?.replicas as PodsCount;
      let replaced = hashes.insert(hash, pods_count);
      assert!(replaced.is_none())

   }

   let version = replicasets
      .metadata
      .resource_version
      .as_ref()
      .ok_or(ClientError::Json("no version in replicasets"))?
      .clone()
      .into();

   Ok(ReplicaSets { hashes, version })
}
