use crate::client::KubeClient;

use tokio::sync::{RwLock, broadcast, mpsc, watch};
use tokio::time::Duration;

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum Metric {
   Inactive,
   Active(NanoCores),
}

pub struct NodeWatcherUtil {
   pub node_names: Arc<RwLock<Vec<Arc<str>>>>,
   pub metric_receivers: Arc<RwLock<Vec<mpsc::Receiver<Metric>>>>,
   pub version: Box<str>,
   pub signal_sender: Arc<RwLock<broadcast::Sender<()>>>,
}

pub type NodeStatusWriters = Arc<RwLock<Vec<watch::Sender<bool>>>>;
pub type NanoCores = u64;

pub struct MetricsCollector {

   client: KubeClient,
   node_names: Arc<RwLock<Vec<Arc<str>>>>,
   nodes_resource_version: Box<str>,

   node_status_receivers: Option<Vec<watch::Receiver<bool>>>,
   node_status_senders: Option<Vec<watch::Sender<bool>>>,


   template_hash: Box<str>,

   pod_uids: Arc<RwLock<HashSet<Box<str>>>>,

   metric_receivers: Arc<RwLock<Vec<mpsc::Receiver<Metric>>>>,
   metric_senders: Option<Vec<mpsc::Sender<Metric>>>,

   pause_duration: Duration,

   signal_sender: Arc<RwLock<broadcast::Sender<()>>>,
}

impl MetricsCollector {
   pub async fn new(
      client: &KubeClient,
      deployment_name: impl AsRef<str>,
      namespace: impl AsRef<str>,
      pause_duration: Duration,
   ) -> Self {
      use crate::initialization::{self, InitialNodes};

      let deployment_name = deployment_name.as_ref();
      let namespace = namespace.as_ref();
      let deployment_uid =
         initialization::get_deployment_uuid(client, namespace, deployment_name).await;

      let (node_names, node_statuses, node_versoin ) = initialization::get_nodes_names(client).await;

      let template_hash = initialization::get_replicaset(client, namespace, &deployment_uid).await;
      let (pod_uids, pod_version) = initialization::get_pods_uids(client, &template_hash).await;

      let (node_status_receivers, node_status_senders) = {
         let mut receivers = Vec::new();
         let mut senders = Vec::new();

         for ready_status in node_statuses {
            let (tx, rx) = watch::channel(ready_status);
            receivers.push(rx);
            senders.push(tx);
         }

         (receivers, senders)
      };

      let (metric_senders, metric_receivers) = {
         let mut senders = Vec::new();
         let mut receivers = Vec::new();

         for _ in node_names.iter() {
            let (sender, receiver) = mpsc::channel::<Metric>(1000);
            senders.push(sender);
            receivers.push(receiver);
         }

         (senders, receivers)
      };


      let node_names = Arc::new(RwLock::new(node_names));
      let nodes_resource_version = resource_version;

      let node_status_receivers = Some(node_status_receivers);
      let node_status_senders = Some(node_status_senders);

      let replica_set_resource_version = resource_version;
      let replica_set_name = name;
      let replica_set_template_hash = pod_template_hash;

      let pod_uids = Arc::new(RwLock::new(pod_uids));

      let metric_receivers = Arc::new(RwLock::new(metric_receivers));
      let metric_senders = Some(metric_senders);

      let client = client.clone();

      let signal_sender = broadcast::Sender::<()>::new(10);
      let signal_sender = Arc::new(RwLock::new(signal_sender));





      Self {
         client,

         node_names,
         nodes_resource_version,
         node_status_senders,
         node_status_receivers,

         replica_set_name,
         replica_set_resource_version,
         replica_set_template_hash,

         pod_uids,

         metric_receivers,
         metric_senders,

         pause_duration,

         signal_sender,
      }
   }

   async fn spawn_node_tasks(&mut self) {
      use super::tasks::node_query_task;

      let node_names = &self.node_names;

      let node_status_receivers = self
         .node_status_receivers
         .take()
         .expect("status receivers should not have been none");

      let metric_senders = self
         .metric_senders
         .take()
         .expect("metric senders should not have been none");

      for ((node_name, status_receiver), metric_sender) in node_names
         .read()
         .await
         .iter()
         .zip(node_status_receivers.into_iter())
         .zip(metric_senders.into_iter())
      {
         let node_name = node_name.clone();

         let signal_receiver = self.signal_sender.read().await.subscribe();
         let pod_uids = self.pod_uids.clone();
         let client = self.client.clone();

         tokio::spawn(node_query_task(client, node_name, pod_uids, status_receiver, signal_receiver, metric_sender));

      }
   }







   




}
