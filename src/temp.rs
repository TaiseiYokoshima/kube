// fn get_client() -> KubeClient {
//    let (host, cert, ident) = {
//       let (host, ca_cert, client_cert, client_key) = read_config();
//       let (cert, ident) = generate_creds(ca_cert, client_cert, client_key);
//
//       (host, cert, ident)
//    };
//
//    let client = Client::builder()
//       .use_rustls_tls()
//       // without this, it didnt use rustls and identity would fail
//       // because identity provided here is not compatible with
//       // native-tls
//       .add_root_certificate(cert)
//       .identity(ident)
//       .build()
//       .unwrap();
//
//    KubeClient::new(client, host)
// }

// fn get_client() -> (reqwest::Client, String) {
//    let (host, cert, ident) = {
//       let (host, ca_cert, client_cert, client_key) = read_config();
//       let (cert, ident) = generate_creds(ca_cert, client_cert, client_key);
//
//       (host, cert, ident)
//    };
//
//    let client = Client::builder()
//       .use_rustls_tls()
//       // without this, it didnt use rustls and identity would fail
//       // because identity provided here is not compatible with
//       // native-tls
//       .add_root_certificate(cert)
//       .identity(ident)
//       .build()
//       .unwrap();
//
//    (client, host)
//
// }






// async fn get_nodes(client: KubeClient) {
//    let url = "/api/v1/nodes";
//    let response = client.get(url).send().await.unwrap();
//    let nodes_list = response.json::<List<Node>>().await.unwrap();
//    let node = nodes_list.items.get(1).unwrap();
//    let data = &node.metadata;
//    let option = &data.name;
//    println!("{:?}", data.name);
// }

// async fn get_nodes_latest_resource_version(client: &KubeClient) -> String {
//    let url = "/api/v1/nodes";
//    let response = client.get(url).send().await.unwrap();
//    let nodes_list = response.json::<List<Node>>().await.unwrap();
//    nodes_list.metadata.resource_version.unwrap()
// }





// pub async fn watch_pods(
//    client: KubeClient,
//    version: Box<str>,
//    template_hash: Box<str>,
// ) {
// use k8s_openapi::{
//    api::core::v1::Pod,
// };
//
// use tokio_util::codec::{FramedRead, LinesCodec};
//    let mut lines = {
//       let url = format!("/api/v1/watch/pods?resourceVersion={}&labelSelector=pod-template-hash%3D{}", version, template_hash);
//       let stream = client
//          .get(url)
//          .send()
//          .await
//          .unwrap()
//          .bytes_stream()
//          .map(|b| b.map_err(std::io::Error::other));
//
//       let reader = tokio_util::io::StreamReader::new(stream);
//       FramedRead::new(reader, LinesCodec::new())
//    };
//
//    while let Some(line) = lines.next().await {
//       let line = line.unwrap();
//       let event: WatchEvent<Pod> = serde_json::from_str(&line).unwrap();
//
//       match event {
//          WatchEvent::Added(pod) => {
//             println!("pod created: {}", pod.metadata.name.unwrap());
//          },
//
//          WatchEvent::Deleted(pod) => {
//             println!("pod deleted: {}", pod.metadata.name.unwrap());
//          },
//          _ => (),
//       };
//    };
//
//    println!("pods watch for hash `{template_hash}` quitting");
// }

