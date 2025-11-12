use k8s_openapi::{List, api::core::v1::Pod as JsonPod};

use crate::client::CAdvisorDaemonSetMetadata;

use super::{APIError, Base, CAdvisorPods, errors, response_into_error, parse_json_pod};


pub async fn get_daemon_set_pods(
   client: &Base,
   daemon_set: &CAdvisorDaemonSetMetadata,
) -> Result<CAdvisorPods, APIError>
{
   let CAdvisorDaemonSetMetadata {
      value,
      key,
      namespace,
   } = daemon_set;

   let endpoint = format!("/api/v1/namespaces/{namespace}/pods?labelSelector={key}={value}");

   let response = {
      let response = client.get(endpoint).send().await?;
      response_into_error(response).await?
   };

   let pods = response.json::<List<JsonPod>>().await?;
   let mut set = Vec::new();

   let version = pods
      .metadata
      .resource_version
      .as_ref()
      .ok_or(errors::RESOURCE_VERSION)?
      .clone()
      .into();

   let pods = pods.items;

   for pod in pods {
      let pod = parse_json_pod(pod, "get")?;
      set.push(pod);
   }

   Ok(CAdvisorPods { pods: set, version })
}
