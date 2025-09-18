use crate::client::KubeClient;
use std::sync::Arc;


pub async fn get_nodes_names(client: &KubeClient) -> (Vec<Arc<str>>, Vec<bool>, Box<str>) {
   use k8s_openapi::{List, api::core::v1::Node};

   let enpoint = "/api/v1/nodes/";
   let response = client
      .get(enpoint)
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();


   let nodes = response.json::<List<Node>>().await.unwrap();

   let version = nodes.metadata.resource_version.as_ref().unwrap();
   let mut names = Vec::new();
   let mut statuses = Vec::new();

   for (i, node) in nodes.items.iter().enumerate() {
      let name = node.metadata.name.as_ref().unwrap();
      let uid = node.metadata.uid.as_ref().unwrap();
      let status_str = node
         .status
         .as_ref()
         .unwrap()
         .conditions
         .as_ref()
         .unwrap()
         .iter()
         .find(|condition| condition.type_ == "Ready")
         .unwrap()
         .status
         .as_str();

      let status = if status_str == "True" {
         true
      } else {
         false
      };

      println!("{i}: name: {name} | uid: {uid} | ready: {status}");
      statuses.push(status);
      names.push(name);
   }

   let names = names
      .into_iter()
      .map(|string| string.clone().into())
      .collect();

   let version = version.clone().into();


   (names, statuses, version)
}
