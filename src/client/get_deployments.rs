use super::deployment::UniqueDeployment;
use super::ClientError;
use super::client::Base;

use std::rc::Rc;

use k8s_openapi::api::apps::v1::Deployment;
use serde::ser::Error;

pub async fn get_deployments(client: &Rc<Base>, deployments: Vec<UniqueDeployment>) -> Result<Vec<UniqueDeployment>, ClientError> {
   let mut errors = Vec::new();

   for deployment in deployments.iter_mut() {
      let namespace = deployment.namespace.as_ref();

      let endpoint = format!("/apis/apps/v1/namespaces/{namespace}/deployments/{deployment}");

      // checks if api send fails
      // checks if the server sent back an error
      let response = client.get(endpoint).send().await?.error_for_status();


      let response = match response {
         Ok(response) => response,
         Err(e) => {
            



         },
      };


      
      
      

         Ok(response) => {
            match response.error_for_status() {
               Ok(response) => response,
               Err(error) => {
                  output.push(Err(error.into()));
                  continue;
               },

            }
         },
         Err(error) => {
            output.push(Err(error.into()));
            continue;
         },
      };

      
      let deployment = match response.json::<Deployment>().await {
         Ok(deployment) => deployment,
         Err(e) => {
            output.push(Err(e.into()));
            continue;
         },
      };


      let uid = deployment.metadata.uid.ok_or(ClientError::Json("no uid in deployment".into()));

      let uid = match uid {
         Ok(uid) =>  uid,
         Err(e) => {
            output.push(Err(e));
            continue;
         },
      };


      let deployment = UniqueDeployment {

      };


   };




      

      

   };



   output
}
