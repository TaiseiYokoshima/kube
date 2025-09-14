use crate::K8SClient;

#[derive(Debug)]
pub struct ReplicaSetTarget {
   pub uid: Box<str>,
   pub name: Box<str>,
   pub app: Box<str>,
   pub pod_template_hash: Box<str>,
}

pub async fn get_replicaset(
   client: &K8SClient,
   namespace: &str,
   deployment_uuid: &str,
) 
   -> ReplicaSetTarget 
{
   use k8s_openapi::{List, api::apps::v1::ReplicaSet};

   let endpoint = format!("/apis/apps/v1/namespaces/{}/replicasets", namespace);
   let response = client
      .get(endpoint)
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();

   let replica_sets = response.json::<List<ReplicaSet>>().await.unwrap();

   let mut replica_set = None;

   for set in replica_sets.items {
      let owners = set.metadata.owner_references.as_ref().unwrap();
      for owner in owners {
         if owner.uid == deployment_uuid && owner.controller.unwrap() {
            replica_set = Some(set);
            break;
         };
      }
   }

   let replica_set = replica_set.expect(&format!(
      "no replica set found for the deployment {deployment_uuid}"
   ));

   let uid = replica_set.metadata.uid.as_ref().unwrap();
   let name = replica_set.metadata.name.as_ref().unwrap();
   let app = replica_set
      .metadata
      .labels
      .as_ref()
      .unwrap()
      .get("app")
      .unwrap();
   let pod_template_hash = replica_set
      .metadata
      .labels
      .as_ref()
      .unwrap()
      .get("pod-template-hash")
      .unwrap();

   ReplicaSetTarget {
      uid: uid.clone().into(),
      name: name.clone().into(),
      app: app.clone().into(),
      pod_template_hash: pod_template_hash.clone().into(),
   }
}
