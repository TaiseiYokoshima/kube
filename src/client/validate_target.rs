use std::rc::Rc;

use super::ClientError;
use super::client::Base;
use super::{Target, TargetInput};
use super::{Kind, TargetError};

use k8s_openapi::{
   List, 
   api::apps::v1::Deployment, 
   api::core::v1::Container
};

fn check_containers<'a>(
   input_containers: impl IntoIterator<Item = &'a Box<str>>,
   deployment: &Deployment,
) -> Result<Vec<usize>, ClientError> {
   let queried_containers: &Vec<Container> = deployment
      .spec
      .as_ref()
      .ok_or(ClientError::Json("no spec in deployment"))?
      .template
      .spec
      .as_ref()
      .ok_or(ClientError::Json("no spec in deployment.spec.template"))?
      .containers
      .as_ref();

   let mut indices = Vec::new();

   let queried: Vec<&str> = queried_containers
      .iter()
      .map(|container| container.name.as_str())
      .collect();

   for (i, to_check) in input_containers.into_iter().enumerate() {
      let found = queried
         .iter()
         .find(|queried| **queried == to_check.as_ref());

      if found.is_none() {
         indices.push(i);
      };
   }

   Ok(indices)
}

fn check_deploymment_name(deployment: &Deployment, to_check: &str) -> Result<bool, ClientError> {
   let deployment_name = deployment
      .metadata
      .name
      .as_ref()
      .ok_or(ClientError::Json("no name in deployment metadata"))?
      .as_str();

   Ok(deployment_name == to_check)
}

fn check_namespace(deployment: &Deployment, to_check: &str) -> Result<bool, ClientError> {
   let namespace = deployment
      .metadata
      .namespace
      .as_ref()
      .ok_or(ClientError::Json("no namespace in deployment metadata"))?
      .as_str();

   Ok(namespace == to_check)
}

fn validate_input_target(
   target: TargetInput,
   deployments: &List<Deployment>,
) -> Result<Result<Target, TargetError>, ClientError> {
   let mut found_namespace = false;

   // iterate the queried deployments in all namespaces to match against the input target
   for deployment in &deployments.items {
      if !check_namespace(&deployment, &target.namespace)? {
         continue;
      };

      found_namespace = true;

      if !check_deploymment_name(&deployment, &target.deployment_name)? {
         continue;
      };

      // if containers are listed
      // check if they are valid
      if !target.containers.is_empty() {
         let error_indicies = check_containers(target.containers.iter(), &deployment)?;

         if !error_indicies.is_empty() {
            let error = TargetError {
               target: target,
               kind: Kind::Containers(error_indicies),
            };

            return Ok(Err(error));
         };
      };

      let deployment_uid = deployment
         .metadata
         .uid
         .as_ref()
         .ok_or(ClientError::Json("no uid in deployment metadata"))?
         .clone()
         .into();


      let namespace = target.namespace;
      let deployment_name = target.deployment_name;
      let mut containers = Some(target.containers);

      if containers.as_ref().unwrap().is_empty() {
         containers = None;
      };

      let target = Target {
         deployment_uid,
         deployment_name,
         namespace,
         containers,
      };

      return Ok(Ok(target));
   }

   let error = if found_namespace {
      let error = TargetError {
         kind: Kind::Deployment,
         target: target,
      };

      Ok(Err(error))
   } else {
      let error = TargetError {
         kind: Kind::Namespace,
         target: target,
      };

      Ok(Err(error))
   };

   error
}

pub async fn validate_targets(
   client: &Rc<Base>,
   input_targets: Vec<TargetInput>,
) -> Result<Result<Vec<Target>, Vec<TargetError>>, ClientError> {
   let endpoint = "/apis/apps/v1/deployments";

   let text = client
      .get(endpoint)
      .send()
      .await?
      .error_for_status()?
      .text()
      .await?;

   let queried_deployments = serde_json::from_str::<List<Deployment>>(&text)?;
   let mut errors = Vec::new();
   let mut targets = Vec::new();

   for input_target in input_targets {
      let result = validate_input_target(input_target, &queried_deployments)?;

      match result {
         Ok(value) => targets.push(value),
         Err(e) => errors.push(e),
      };
   }

   let output = if errors.is_empty() {
      Ok(targets)
   } else {
      Err(errors)
   };

   Ok(output)
}
