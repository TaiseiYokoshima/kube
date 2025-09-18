use std::sync::Arc;

use super::super::{Metric, NanoCores};

use futures::future::join_all;
use tokio::sync::{RwLock, mpsc};

pub async fn metrics_collector_task(
   metric_receivers: Arc<RwLock<Vec<mpsc::Receiver<Metric>>>>,
) -> Vec<NanoCores> {
   let mut output = Vec::new();

   loop {
      let mut cpu = 0;

      {
         let mut lock = metric_receivers.write().await;
         let len = lock.len();
         if len == 0 {
            break;
         }

         let results = join_all(lock.iter_mut().map(|receiver| receiver.recv())).await;

         let mut i: usize = 0;
         lock.retain(|_| {
            let result = results
               .get(i)
               .expect("For some reason the vec size has changed")
               .as_ref();

            i += 1;

            let value = match result {
               None => return false,
               Some(value) => value,
            };

            match value {
               Metric::Inactive => (),
               Metric::Active(value) => cpu += *value,
            };

            true
         });

         drop(lock);
      };
      output.push(cpu);
   }

   output
}
