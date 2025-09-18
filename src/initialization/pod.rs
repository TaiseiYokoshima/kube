use std::collections::HashSet;

use crate::client::KubeClient;

pub async fn get_pods_uids(client: &KubeClient, template_hash: &str) -> (Box<str>, HashSet<Box<str>>) {
   use k8s_openapi::{List, api::core::v1::Pod};

   let endpoint =
      format!("/api/v1/namespaces/default/pods?labelSelector=pod-template-hash={template_hash}");
   let response = client
      .get(endpoint)
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
   let pods = response.json::<List<Pod>>().await.unwrap();

   let mut uids = HashSet::new();

   for pod in pods.items.iter() {
      let uid = pod.metadata.uid.as_ref().unwrap();
      uids.insert(uid);
   }

   let uids = uids
      .into_iter()
      .map(|string| string.clone().into())
      .collect();

   let version = pods.metadata.resource_version.unwrap().into();

   (version, uids)
}
