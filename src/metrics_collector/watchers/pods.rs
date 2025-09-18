use std::{collections::HashSet, sync::Arc};

use tokio::sync::RwLock;

use tokio_util::codec::{FramedRead, LinesCodec};
use futures::stream::StreamExt;


use crate::client::KubeClient;

use k8s_openapi::{
   List,
   apimachinery::pkg::apis::meta::v1::WatchEvent,
   api::core::v1::Pod,
};

// pub async fn watch_pods(
//    client: KubeClient,
//    version: Box<str>,
//    pod_uids: Arc<RwLock<Vec<Box<str>>>>,
//    template_hash: Box<str>,
// ) {
//    let mut lines = {
//       let url = format!("/api/v1/watch/pods?resourceVersion={}&pod-template-hash={}", version, template_hash);
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
//       let line = line.expect("no line");
//       let event: WatchEvent<Node> = serde_json::from_str(&line).unwrap();
//
//       match event {
//          WatchEvent::Added(node) => {
//
//             let conditions = &node.status.unwrap().conditions.unwrap();
//             let status = *&conditions.iter().find(|c| c.type_ == "Ready").unwrap().status.to_lowercase().parse::<bool>().unwrap();
//             let name = node.metadata.name.unwrap().into();
//             
//
//             let status_sender = add_node(&client, name, status, &node_names, &metric_receivers, &pod_uids, &signal_sender).await;
//             status_senders.push(status_sender);
//          },
//
//          WatchEvent::Deleted(node) => {
//             let name = node.metadata.name.unwrap();
//             let i = index(&name, &node_names).await.unwrap();
//             node_names.write().await.remove(i);
//             status_senders.remove(i);
//          },
//
//          WatchEvent::Modified(node) => {
//             let conditions = &node.status.unwrap().conditions.unwrap();
//             let status = *&conditions.iter().find(|c| c.type_ == "Ready").unwrap().status.to_lowercase().parse::<bool>().unwrap();
//             let name = node.metadata.name.unwrap();
//             let i = index(&name, &node_names).await.unwrap();
//
//             let _ = status_senders.get(i).unwrap().send(status);
//          },
//
//          other => {
//             println!("other: {:?}", other);
//          }
//
//       }
//
//    };
//
//    println!("no more events");
//
// }


pub async fn watch_pods(
   client: KubeClient,
   version: Box<str>,
   template_hash: Box<str>,
   pod_uids: Arc<RwLock<HashSet<Box<str>>>>,
) {
   let mut lines = {
      let url = format!("/api/v1/watch/pods?resourceVersion={}&pod-template-hash={}", version, template_hash);
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
      let line = line.unwrap();
      let event: WatchEvent<Pod> = serde_json::from_str(&line).unwrap();

      match event {
         WatchEvent::Added(pod) => {
            println!("pod created: {}", pod.metadata.name.unwrap());
            pod_uids.write().await.insert(pod.metadata.uid.unwrap().into());
         },

         WatchEvent::Deleted(pod) => {
            println!("pod deleted: {}", pod.metadata.name.unwrap());
            let uid = pod.metadata.uid.unwrap().into_boxed_str();
            pod_uids.write().await.remove(&uid);
         },
         _ => (),
      };
   };

   println!("pods watch for hash `{template_hash}` quitting");
}
