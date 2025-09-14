mod k8s_client;
mod metrics_collector;


mod initialization;


use metrics_collector::extract_cpu_metrics;


use crate::k8s_client::K8SClient;
use futures_util::StreamExt;
use reqwest::{Certificate, Client, Identity};
use tokio_util::codec::{FramedRead, LinesCodec};

use k8s_openapi::{api::core::v1::Node, apimachinery::pkg::apis::meta::v1::WatchEvent, serde_json, List};

fn read_config() -> (String, String, String, String) {
   use serde_yaml::Value;
   use std::fs;

   let user = std::env::var("USER").unwrap();
   let path = format!("/home/{}/.kube/config", user);
   let s = fs::read_to_string(path).unwrap();
   let config: Value = serde_yaml::from_str(&s).unwrap();

   let first_cluster = config
      .get("clusters")
      .expect("no clusters")
      .as_sequence()
      .expect("not sequence")
      .get(0)
      .expect("empty sequence")
      .get("cluster")
      .expect("no cluster");

   let host = first_cluster
      .get("server")
      .expect("no server")
      .as_str()
      .expect("not str");

   let ca_cert = first_cluster
      .get("certificate-authority-data")
      .expect("no ca")
      .as_str()
      .expect("not str");

   let client_credentials = config
      .get("users")
      .expect("no users")
      .as_sequence()
      .expect("not sequences")
      .get(0)
      .expect("empty sequence")
      .get("user")
      .expect("no user");

   let client_cert = client_credentials
      .get("client-certificate-data")
      .expect("no client ca")
      .as_str()
      .expect("not str");

   let client_key = client_credentials
      .get("client-key-data")
      .expect("no client key")
      .as_str()
      .expect("not str");

   let host = host.into();
   let ca_cert = ca_cert.into();
   let client_cert = client_cert.into();
   let client_key = client_key.into();

   (host, ca_cert, client_cert, client_key)
}

fn generate_creds(
   ca_cert: String,
   client_cert: String,
   client_key: String,
) -> (Certificate, Identity) {
   use base64::{Engine, engine::general_purpose::STANDARD};

   let ca_cert = STANDARD.decode(ca_cert).unwrap();
   let client_cert = STANDARD.decode(client_cert).unwrap();
   let client_key = STANDARD.decode(client_key).unwrap();

   let ca_cert = Certificate::from_pem(&ca_cert).unwrap();

   let mut pem = vec![];
   pem.extend_from_slice(&client_cert);
   pem.extend_from_slice(&client_key);

   let identity = Identity::from_pem(&pem).unwrap();

   (ca_cert, identity)
}

fn get_client() -> K8SClient {
   let (host, cert, ident) = {
      let (host, ca_cert, client_cert, client_key) = read_config();
      let (cert, ident) = generate_creds(ca_cert, client_cert, client_key);

      (host, cert, ident)
   };

   let client = Client::builder()
      .use_rustls_tls()
      // without this, it didnt use rustls and identity would fail
      // because identity provided here is not compatible with
      // native-tls
      .add_root_certificate(cert)
      .identity(ident)
      .build()
      .unwrap();

   K8SClient::new(client, host)
}

async fn get_nodes(client: K8SClient) {
   let url = "/api/v1/nodes";
   let response = client.get(url).send().await.unwrap();
   let nodes_list = response.json::<List<Node>>().await.unwrap();
   let node = nodes_list.items.get(1).unwrap();
   let data = &node.metadata;
   let option = &data.name;
   println!("{:?}", data.name);
}

async fn get_nodes_latest_resource_version(client: &K8SClient) -> String {
   let url = "/api/v1/nodes";
   let response = client.get(url).send().await.unwrap();
   let nodes_list = response.json::<List<Node>>().await.unwrap();
   nodes_list.metadata.resource_version.unwrap()
}




async fn watch_nodes(client: K8SClient, version: String) {
   let mut lines = {
      let url = format!("/api/v1/watch/nodes?resourceVersion={}", version);
      let stream = client
         .get(url)
         .send()
         .await
         .unwrap()
         .bytes_stream()
         .map(|b| b.map_err(std::io::Error::other));

      let reader = tokio_util::io::StreamReader::new(stream);
      FramedRead::new(reader, LinesCodec::new())
   };

   while let Some(line) = lines.next().await {
      let line = line.expect("no line");
      let event: WatchEvent<Node> = serde_json::from_str(&line).unwrap();

      match event {
         WatchEvent::Added(node) => {

            let conditions = &node.status.unwrap().conditions.unwrap();
            let status = &conditions.iter().find(|c| c.type_ == "Ready").unwrap().status.to_lowercase().parse::<bool>().unwrap();
            println!("`{}` created | ready: {}", node.metadata.name.unwrap(), status);
            // println!("{:?}", node.)

         },
         WatchEvent::Deleted(node) => {
            println!("`{}` deleted", node.metadata.name.unwrap());

         },
         WatchEvent::Modified(node) => {
            println!("`{}` modified", node.metadata.name.unwrap());
         },

         other => {
            println!("other: {:?}", other);
         }

      }

   }

   println!("no more events");
}


static DEPLOYMENT: &'static str = "stress-test";

#[tokio::main]
async fn main() {
   use initialization::ReplicaSetTarget;

   let client = get_client();

   let deployment_uuid = initialization::get_deployment_uuid(&client, "default", DEPLOYMENT).await;
   let replicaset = initialization::get_replicaset(&client, "default", &deployment_uuid).await;

   let pods = initialization::get_pods_uids(&client, &replicaset.pod_template_hash).await;


   // println!("replicaset {replicaset:?}");
   // println!("uuid: {deployment_uuid}");
   return


   // let version = get_nodes_latest_resource_version(&client).await;



   // watch_nodes(client, version).await;

}
