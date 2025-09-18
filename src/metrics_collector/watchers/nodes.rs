use std::sync::Arc;

use crate::client::KubeClient;
use crate::metrics_collector::Metric;

use tokio::sync::{
   RwLock,
   broadcast,
   watch,
   mpsc,
};


use tokio_util::codec::{FramedRead, LinesCodec};


use k8s_openapi::{
   apimachinery::pkg::apis::meta::v1::WatchEvent,
   api::core::v1::Node,
};

use futures::stream::StreamExt;


use super::super::tasks::node_query_task;

async fn add_node(
   client: &KubeClient,
   node_name: Arc<str>, 
   status: bool, 
   node_names: &Arc<RwLock<Vec<Arc<str>>>>,
   metric_receivers: &Arc<RwLock<Vec<mpsc::Receiver<Metric>>>>,
   pod_uids: &Arc<RwLock<Vec<Box<str>>>>,
   signal_sender: &Arc<RwLock<broadcast::Sender<()>>>,
   ) -> watch::Sender<bool> {
   let (metric_sender, metric_receiver) = mpsc::channel(100);

   {
      metric_receivers.write().await.push(metric_receiver);
   };

   {
      node_names.write().await.push(node_name.clone());

   };

   let (status_sender, status_receiver) = watch::channel(status);

   let signal_receiver = signal_sender.read().await.subscribe();
   let pod_uids = pod_uids.clone();

   let client = client.clone();

   tokio::spawn(node_query_task(client, node_name, pod_uids, status_receiver, signal_receiver, metric_sender));

   status_sender
}



async fn index(name: &str, vec: &Arc<RwLock<Vec<Arc<str>>>>) -> Option<usize> {
   vec.read().await.iter().position(|current| current.as_ref() == name)
}




pub async fn watch_nodes(
   client: KubeClient, 
   version: String, 
   node_names: Arc<RwLock<Vec<Arc<str>>>>,
   metric_receivers: Arc<RwLock<Vec<mpsc::Receiver<Metric>>>>,
   pod_uids: Arc<RwLock<Vec<Box<str>>>>,
   signal_sender: Arc<RwLock<broadcast::Sender<()>>>,
   mut status_senders: Vec<watch::Sender<bool>>,
) {
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
            let status = *&conditions.iter().find(|c| c.type_ == "Ready").unwrap().status.to_lowercase().parse::<bool>().unwrap();
            let name = node.metadata.name.unwrap().into();
            

            let status_sender = add_node(&client, name, status, &node_names, &metric_receivers, &pod_uids, &signal_sender).await;
            status_senders.push(status_sender);
         },

         WatchEvent::Deleted(node) => {
            let name = node.metadata.name.unwrap();
            let i = index(&name, &node_names).await.unwrap();
            node_names.write().await.remove(i);
            status_senders.remove(i);
         },

         WatchEvent::Modified(node) => {
            let conditions = &node.status.unwrap().conditions.unwrap();
            let status = *&conditions.iter().find(|c| c.type_ == "Ready").unwrap().status.to_lowercase().parse::<bool>().unwrap();
            let name = node.metadata.name.unwrap();
            let i = index(&name, &node_names).await.unwrap();

            let _ = status_senders.get(i).unwrap().send(status);
         },

         other => {
            println!("other: {:?}", other);
         }

      }

   };

   println!("no more events");
}


