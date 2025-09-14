use crate::K8SClient;

pub async fn get_deployment_uuid(
   client: &K8SClient,
   namespace: &str,
   deployment_name: &str,
) -> String {
   use k8s_openapi::api::apps::v1::Deployment;

   let endpoint = format!(
      "/apis/apps/v1/namespaces/{}/deployments/{}",
      namespace, deployment_name
   );
   let response = client
      .get(endpoint)
      .send()
      .await
      .unwrap()
      .error_for_status()
      .unwrap();
   let deployment = response.json::<Deployment>().await.unwrap();

   deployment.metadata.uid.unwrap()
}
