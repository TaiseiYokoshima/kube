use std::sync::Arc;

use crate::client::KubeClient;

use super::super::{Metric, tasks::node_json_parse_task};

use reqwest::Response;
use tokio::sync::{RwLock, broadcast, mpsc, watch};

pub async fn node_query_task(
   client: KubeClient,
   node_name: Arc<str>,
   pod_uids: Arc<RwLock<Vec<Box<str>>>>,
   mut status_receiver: watch::Receiver<bool>,
   mut signal_receiver: broadcast::Receiver<()>,
   metric_sender: mpsc::Sender<Metric>,
) {
   let (response_sender, response_receiver) = mpsc::channel::<Option<Response>>(100);
   tokio::task::spawn(node_json_parse_task(
      response_receiver,
      metric_sender,
      node_name.clone(),
      pod_uids,
   ));

   loop {
      {
         let result = signal_receiver.recv().await;
         if result.is_err() {
            println!(
               "metric query task quitting due to metric signal sender dropped | node: {node_name}"
            );
            return;
         };

         match status_receiver.has_changed() {
            Ok(value) => value,
            _ => {
               println!(
                  "metric query task quitting due to ready status sender dropped | node: {node_name}"
               );
               return;
            }
         };

         let ready = *status_receiver.borrow_and_update();

         if !ready {
            let result = response_sender.send(None).await;

            if result.is_err() {
               println!(
                  "metric query task quitting due to json parser response receiver dropped | node: {node_name}"
               );
               return;
            };
         };
      };

      let endpoint = format!("/api/v1/nodes/{node_name}/proxy/stats/summary");

      let response = {
         let mut server_error: u8 = 1;
         loop {
            // checks if api requets can be made with `tries` times attempts
            let mut tries: u8 = 1;
            let response = loop {
               let response = client.get(&endpoint).send().await;

               match response {
                  Ok(value) => break value,
                  Err(e) => {
                     println!(
                        "{tries}: query task api request failed due to {e} | node: {node_name}"
                     );

                     if tries == 10 {
                        println!(
                           "query task quitting due to too many api request failures | node: {node_name}"
                        );
                        return;
                     };

                     tries += 1;
                  }
               }
            };

            let response = response.error_for_status();

            // checks if server responds in valid response in `server_error` attempts
            match response {
               Ok(response) => break response,
               Err(e) => {
                  println!(
                     "{server_error}: query task api request received server error: {e} | node: {node_name}"
                  );

                  if server_error == 10 {
                     println!(
                        "query task quitting due to too many error response from the server | node: {node_name}"
                     );
                     return;
                  };

                  server_error += 1;
               }
            };
         }
      };

      let result = response_sender.send(Some(response)).await;

      if result.is_err() {
         println!(
            "metric query task quitting due to json parser response receiver dropped | node: {node_name}"
         );
         return;
      };
   }
}
