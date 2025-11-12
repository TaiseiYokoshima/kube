use super::{Pod, APIError, errors};
use k8s_openapi::{
   api::core::v1::Pod as JsonPod,
   apimachinery::pkg::apis::meta::v1::ObjectMeta,
};


pub fn parse_json_pod(pod: JsonPod, context: &str) -> Result<Pod, APIError> {

   let JsonPod {
      metadata,
      status,
      ..
   } = pod;

   let ObjectMeta {
      name,
      namespace,
      uid,
      ..
   } = metadata;

   let name = name.ok_or(errors::NAME)?;
   let uid = uid.ok_or(errors::UID)?;
   let namespace = namespace.ok_or(errors::NAMESPACE)?;
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
            .ok_or(errors::READY_CONDITION)
      }) {
      Ok(status) => status,
      Err(e) => {
         println!("got {e:?} in {context} but fell back to false");
         false
      }
   };
   
   Ok(Pod::new(uid.into(), namespace.into(), name.into(), status))
}
