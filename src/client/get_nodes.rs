use std::rc::Rc;

use k8s_openapi::{List, api::core::v1::Node};

use super::Nodes;
use super::{ClientError, client::Base};

fn get_status(node: &Node) -> Result<bool, ClientError> {
   let conditions = node
      .status
      .as_ref()
      .ok_or(ClientError::Json("no status in node"))?
      .conditions
      .as_ref()
      .ok_or(ClientError::Json("no conditions in node's status"))?;

   let status_str = conditions
      .iter()
      .find(|condition| condition.type_ == "Ready")
      .ok_or(ClientError::Json(
         "no ready condition in nodes' status' conditions",
      ))?
      .clone()
      .status;

   let status = if status_str == "True" { true } else { false };

   Ok(status)
}

pub async fn get_nodes(client: &Base) -> Result<Nodes, ClientError> {
   let endpoint = "/api/v1/nodes/";

   let response = client
      .get(endpoint)
      .send()
      .await?
      .error_for_status()?
      .text()
      .await?;

   let nodes = serde_json::from_str::<List<Node>>(&response)?;

   let mut names = Vec::new();
   let mut statuses = Vec::new();

   for node in nodes.items.iter() {
      let name = node
         .metadata
         .name
         .as_ref()
         .ok_or(ClientError::Json("no name in node's metadata"))?
         .clone()
         .into();

      let status = get_status(node)?;

      names.push(name);
      statuses.push(status);
   }

   let version = nodes
      .metadata
      .resource_version
      .ok_or(ClientError::Json("no version in nodes' metadata"))?
      .into();

   Ok(Nodes {
      names,
      statuses,
      version,
   })
}
