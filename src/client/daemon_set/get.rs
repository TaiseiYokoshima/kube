use k8s_openapi::{List, api::core::v1::Pod, apimachinery::pkg::apis::meta::v1::ObjectMeta};

use crate::client::CAdvisorDaemonSetMetadata;

use super::{APIError, Base, CAdvisorPods, Pod as ThisPod, errors, response_into_error};

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

   let pods = response.json::<List<Pod>>().await?;
   let mut set = Vec::new();







   let version = pods
      .metadata
      .resource_version
      .as_ref()
      .ok_or(errors::ERR_RESOURCE)?
      .clone()
      .into();

   let pods = pods.items;

   for pod in pods {
      let Pod {
         metadata, status, ..
      } = pod;

      let ObjectMeta { uid, name, .. } = metadata;

      let uid = match uid {
         Some(uid) => uid,
         _ => return Err(errors::UID),
      };

      let name = match name {
         Some(name) => name,
         _ => return Err(errors::NAME),
      };

      let status = match status
         .ok_or(errors::STATUS)
         .and_then(|x| x.conditions.ok_or(errors::CONDITION))
         .and_then(|x| {
            x.iter()
               .find_map(|status| {
                  if status.type_ != "Ready" {
                     None
                  } else {
                     if status.status == "True" {
                        Some(true)
                     } else {
                        Some(false)
                     }
                  }
               })
               .ok_or(errors::READY)
         }) {
         Ok(status) => status,
         Err(e) => {
            println!("got {e:?} in get but fell back to false");
            false
         }
      };

      let pod = ThisPod::new(uid.into(), name.into(), status);
      set.push(pod);
   }

   Ok(CAdvisorPods { pods: set, version })
}
