use crate::K8SClient;

pub async fn get_pods_uids(client: &K8SClient, template_hash: &str) -> Vec<Box<str>> {
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

   let mut uids = Vec::new();

   for (i, pod) in pods.items.iter().enumerate() {
      let name = pod.metadata.name.as_ref().unwrap();
      let uid = pod.metadata.uid.as_ref().unwrap();
      uids.push(uid);
      println!("{i}: name: {} | uid: {}", name, uid);
   }

   let uids: Vec<Box<str>> = uids
      .into_iter()
      .map(|string| string.clone().into_boxed_str())
      .collect();
   uids
}
