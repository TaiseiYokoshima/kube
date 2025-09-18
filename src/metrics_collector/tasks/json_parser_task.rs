use std::sync::Arc;

use super::super::{NanoCores, Metric};

use reqwest::Response;
use tokio::sync::{RwLock, mpsc};

pub async fn node_json_parse_task(
   mut response_receiver: mpsc::Receiver<Option<Response>>,
   metric_sender: mpsc::Sender<Metric>,
   node_name: Arc<str>,
   pod_uids: Arc<RwLock<Vec<Box<str>>>>,
) {
   use serde_json::Value;

   loop {
      let response = match response_receiver.recv().await {
         Some(Some(response)) => response,
         None => {
            println!("json parser task quitting due to sender dropped | node: {node_name}");
            return;
         }
         Some(None) => {
            let result = metric_sender.send(Metric::Inactive).await;

            if result.is_err() {
               println!("json parser task quitting due to metric receiver dropped | node: {node_name}");
               return;
            };

            continue;
         }
      };

      let json = response.json::<Value>().await.unwrap();
      let pods = json.get("pods").unwrap().as_array().unwrap();
      let mut cpu: NanoCores = 0;

      for pod in pods {
         let uid = pod
            .get("podRef")
            .unwrap()
            .get("uid")
            .unwrap()
            .as_str()
            .unwrap();

         {
            let pod_uids = pod_uids.read().await;

            let opt = pod_uids.iter().find(|string| string.as_ref() == uid);

            if opt.is_none() {
               continue;
            };
         };

         let current_cpu = pod
            .get("cpu")
            .unwrap()
            .get("usageNanoCores")
            .unwrap()
            .as_u64()
            .unwrap();

         cpu += current_cpu;
      }

      let result = metric_sender.send(Metric::Active(cpu)).await;

      if result.is_err() {
         println!("json parser task quitting due to receiver dropped | node: {node_name}");
         return;
      }
   }
}
