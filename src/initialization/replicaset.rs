use crate::client::KubeClient;

pub async fn get_replicaset(
   client: &KubeClient,
   namespace: &str,
   deployment_uid: &str,
) 
   -> Box<str>
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
         if owner.uid == deployment_uid && owner.controller.unwrap() {
            replica_set = Some(set);
            break;
         };
      }
   }

   let replica_set = replica_set.expect(&format!(
      "no replica set found for the deployment {deployment_uid}"
   ));


   let pod_template_hash = replica_set
      .metadata
      .labels
      .as_ref()
      .unwrap()
      .get("pod-template-hash")
      .unwrap();

   pod_template_hash.clone().into()
}
