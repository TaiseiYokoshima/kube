use std::collections::{HashMap, HashSet};
use std::time::Duration;

use futures::FutureExt;

use tokio::{
   sync::{mpsc, oneshot},
   task::JoinHandle,
};

use crate::client::{
   CAdvisorDaemonSetMetadata, CAdvisorPods, DaemonSetEvent, EventKind, KubeClient, WatcherError,
};

use crate::metrics::node::{NodeMetric, NodeMetricCollector};
use crate::metrics::querier::TopLevelMetric;

use super::querier::QueryTask;

#[derive(Debug)]
pub struct ScrapeResult
{
   collector_map: HashMap<String, NodeMetricCollector>,
   cpu_percentages: Vec<f64>,
   timestamps: Vec<f64>,
}

fn handle_event(
   event: Result<DaemonSetEvent, WatcherError>,
   metric_sender: &mpsc::Sender<NodeMetric>,
   client: &KubeClient,
   collector_map: &mut HashMap<String, NodeMetricCollector>,
   running_querier_map: &mut HashMap<String, QueryTask>,
   paused_querier_map: &mut HashMap<String, QueryTask>,
)
{
   match event {
      Ok(event) => {
         println!("got event from watcher: {:?} from {}", event.kind, event.pod.name);
         match event.kind {
            EventKind::Created => {
               let querier = QueryTask::new(&event.pod, metric_sender.clone(), client);
               let collector = NodeMetricCollector::new();
               assert!(
                  collector_map
                     .insert(event.pod.uid.clone().into(), collector)
                     .is_none()
               );

               if event.pod.status {
                  assert!(
                     running_querier_map
                        .insert(event.pod.uid.clone().into(), querier)
                        .is_none()
                  );
               } else {
                  assert!(
                     paused_querier_map
                        .insert(event.pod.uid.clone().into(), querier)
                        .is_none()
                  );
               };
            }
            EventKind::Paused => {
               let uid: String = event.pod.uid.clone().into();
               let querier = running_querier_map.remove(&uid).unwrap();
               querier.pause();
               assert!(paused_querier_map.insert(uid, querier).is_none());
            }
            EventKind::Resumed => {
               let uid: String = event.pod.uid.clone().into();
               let querier = paused_querier_map.remove(&uid).unwrap();
               querier.resume();
               assert!(running_querier_map.insert(uid, querier).is_none());
            }
            EventKind::Deleted => {
               let uid: &String = &event.pod.uid.clone().into();
               let running_removed = running_querier_map.remove(uid);
               let paused_removed = paused_querier_map.remove(uid);
               assert_ne!(running_removed.is_some(), paused_removed.is_some());
            }
         }
      }
      Err(e) => {
         println!("Error in metric collector and reading from watcher: {e:?}");
      }
   };
}

async fn scrape(
   client: KubeClient,
   daemon_set_meta: CAdvisorDaemonSetMetadata,
   daemon_set_state: CAdvisorPods,
   killed: oneshot::Receiver<()>,
   // cpu_count: u32,
) -> ScrapeResult
{
   let killed = killed.shared();

   let mut round_set = HashSet::new();
   let mut round_min = None;

   let mut collector_map = HashMap::new();
   let mut running_querier_map: HashMap<String, QueryTask> = HashMap::new();
   let mut paused_querier_map: HashMap<String, QueryTask> = HashMap::new();
   let mut cpu_percentages = Vec::new();
   let mut timestamps = Vec::new();

   let (metric_sender, mut metric_receiver) = mpsc::channel(100);

   let CAdvisorPods { pods, .. } = &daemon_set_state;

   for pod in pods {
      let querier = QueryTask::new(&pod, metric_sender.clone(), &client);
      let collector = NodeMetricCollector::new();
      assert!(
         collector_map
            .insert(pod.uid.clone().into(), collector)
            .is_none()
      );

      if pod.status {
         assert!(
            running_querier_map
               .insert(pod.uid.clone().into(), querier)
               .is_none()
         );
      } else {
         assert!(
            paused_querier_map
               .insert(pod.uid.clone().into(), querier)
               .is_none()
         );
      };
   }

   let mut watcher =
      client
         .watch
         .daemon_set_pods(daemon_set_meta, daemon_set_state, Duration::from_secs(60));

   loop {
      let data_point = tokio::select! {
         _ = killed.clone() => {
            println!("metric collector killed");
            break;
         },
         event = watcher.next() => {
            handle_event(event, &metric_sender, &client, &mut collector_map, &mut running_querier_map, &mut paused_querier_map);
            continue;
         },
         data_point = metric_receiver.recv() => match data_point {
            Some(data) => data,
            None => {
               println!("for some reason all senders of metric queriers are dropped");
               break;
            },
         },
      };

      let NodeMetric { uid, metric } = data_point;
      let TopLevelMetric {
         value: cpu,
         timestamp: time,
      } = metric;

      let collector = collector_map.get_mut(&uid).unwrap();
      let (time, _) = match collector.next(time, cpu) {
         None => continue,
         Some(x) => x,
      };

      round_set.insert(uid);
      match &round_min {
         None => round_min = Some(time),
         Some(min) => {
            if time < *min {
               round_min = Some(time);
            }
         }
      };

      if round_set.len() >= running_querier_map.len() {
         // println!("round complete. computing total value");
         round_set.clear();
         let time = std::mem::replace(&mut round_min, None).unwrap();
         let total_cpu: f64 = collector_map
            .values()
            .map(|passed| passed.interporlate(time))
            .sum();

         // let total_cpu = total_cpu / cpu_count as f64;

         println!("total cpu: {total_cpu}% @ {time}");
         timestamps.push(time);
         cpu_percentages.push(total_cpu);
      };
   }

   let killed_futures: Vec<_> = running_querier_map
      .into_values()
      .map(|queier| queier.kill())
      .chain(
         paused_querier_map
            .into_values()
            .map(|querier| querier.kill()),
      )
      .collect();
   futures::future::join_all(killed_futures).await;

   ScrapeResult {
      collector_map,
      timestamps,
      cpu_percentages,
   }
}

#[derive(Debug)]
pub struct MetricCollector
{
   handle: JoinHandle<ScrapeResult>,
   killer: oneshot::Sender<()>,
   // cpu_count: u32,
}

impl MetricCollector
{
   pub fn new(
      client: KubeClient,
      daemon_set_meta: CAdvisorDaemonSetMetadata,
      daemon_set_state: CAdvisorPods,
      // cpu_count: u32,
   ) -> Self
   {
      let (killer, killed) = oneshot::channel();
      let handle = tokio::spawn(scrape(
         client,
         daemon_set_meta,
         daemon_set_state,
         killed,
         // cpu_count,
      ));
      Self {
         handle,
         killer,
         // cpu_count,
      }
   }

   pub async fn kill(self) -> ScrapeResult
   {
      match self.killer.send(()) {
         Ok(_) => (),
         Err(_) => println!("Error from killing metric collector task"),
      };

      self.handle.await.unwrap()
   }
}
