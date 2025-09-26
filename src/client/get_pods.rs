use super::{ClientError, NamespaceMap, PodSet, Pods, ReplicaSets, client::Base};

use k8s_openapi::{List, api::core::v1::Pod};
use std::{collections::HashMap, rc::Rc};

fn insert_pods(namespace_map: &mut NamespaceMap, pods: List<Pod>) -> Result<(), ClientError> {
   for pod in pods.items {
      let namespace = pod
         .metadata
         .namespace
         .as_ref()
         .ok_or(ClientError::Json("no namespace in pod's metadata"))?
         .clone()
         .into();

      let pod_name = pod
         .metadata
         .name
         .as_ref()
         .ok_or(ClientError::Json("no name in pod's metadata"))?
         .clone()
         .into();

      let pod_set = namespace_map
         .entry(namespace)
         .or_insert_with_key(|_| PodSet::new());

      let was_new = pod_set.insert(pod_name);
      assert!(was_new);
   }

   Ok(())
}

pub async fn get_pods(client: &Base, replica_sets: &ReplicaSets) -> Result<Pods, ClientError> {
   let mut namespace_map = NamespaceMap::new();
   let mut versions = HashMap::new();
   for (hash, count) in replica_sets.hashes.iter() {
      if *count == 0 {
         continue;
      };
      
      let endpoint = format!("/api/v1/pods?labelSelector=pod-template-hash={hash}");

      let body_string = client
         .get(endpoint)
         .send()
         .await?
         .error_for_status()?
         .text()
         .await?;

      let pods = serde_json::from_str::<List<Pod>>(&body_string)?;

      let version = pods
         .metadata
         .resource_version
         .as_ref()
         .ok_or(ClientError::Json("no version in pods' metadata"))?
         .clone()
         .into();

      let replaced = versions.insert(hash.clone(), version).is_some();
      assert!(!replaced);
         

      insert_pods(&mut namespace_map, pods)?;
   }

   let map = namespace_map;
   let versions = versions;

   let pods = Pods { map, versions };

   Ok(pods)
}
