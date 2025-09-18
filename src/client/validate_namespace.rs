use super::client::Base;
use super::ClientError;

use std::rc::Rc;

use k8s_openapi::{
   List,
   api::core::v1::Namespace,
};

pub async fn validate_namespaces(client: &Rc<Base>, namespaces_to_check: impl Iterator<Item = &str>) -> Result<Vec<bool>, ClientError> {
   let endpoint = format!("/api/v1/namespaces");
   let response = client.get(endpoint).send().await?.error_for_status()?.text().await?;

   let namespaces = serde_json::from_str::<List<Namespace>>(&response)?;

   if namespaces.items.is_empty() {
      return Ok(namespaces_to_check.map(|_| false).collect());
   };

   let namespaces: Result<Vec<&str>, ClientError> = namespaces.items.iter().map(|namespace| {
      let name = namespace.metadata.name.as_ref().ok_or(ClientError::Json("no name in namespace metadata"))?.as_str();
      Ok(name)
   }).collect();

   let namespaces = namespaces?;



   let results = namespaces_to_check.map(|namespace_to_check| {
      let exists = namespaces.iter().find(|namespace| {
         **namespace == namespace_to_check 
      });

      exists.is_some()
   }).collect();

   Ok(results)
}
