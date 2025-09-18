use super::{ClientError, client::Base};
use std::{rc::Rc, sync::Arc};


use k8s_openapi::{
   List,
   api::core::v1::Node,
};

pub async fn get_nodes(client: &Rc<Base>) -> Result<(Vec<Arc<str>>, Vec<bool>, Box<str>), ClientError> {
   let endpoint = "/api/v1/nodes/";
   let response = client.get(endpoint).send().await;

   let response = response?.error_for_status()?;

   let nodes = response.json::<List<Node>>().await?;

   let reason = "no version in nodes";
   let version = nodes
      .metadata
      .resource_version
      .as_ref()
      .ok_or(ClientError::Json(reason.into()))?;

   let mut names = Vec::new();
   let mut statuses = Vec::new();

   for node in nodes.items.iter() {
      let reason = "no name in node";
      let name = node
         .metadata
         .name
         .as_ref()
         .ok_or(ClientError::Json(reason.into()))?;

      let status_str = node
         .status
         .as_ref()
         .ok_or(ClientError::Json("no node status in node".into()))?
         .conditions
         .as_ref()
         .ok_or(ClientError::Json("no node condition in node".into()))?
         .iter()
         .find(|condition| condition.type_ == "Ready")
         .ok_or(ClientError::Json(
            "no ready condition in node conditions".into(),
         ))?
         .status
         .as_str();

      let status = if status_str == "True" { true } else { false };

      statuses.push(status);
      names.push(name);
   }

   let names = names
      .into_iter()
      .map(|string| string.clone().into())
      .collect();

   let version = version.clone().into();

   Ok((names, statuses, version))
}
