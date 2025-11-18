use prom_text_format_parser::{Scrape, Value};

use tokio::{
   task::JoinHandle,
  sync::watch,
};

use crate::client::{Pod, KubeClient, APIError};

use super::node::NodeMetric;

#[derive(Debug, Clone, Copy)]
pub enum State
{
   Running,
   Paused,
   Killed,
}

#[derive(Debug)]
pub struct QueryTask
{
   handle: JoinHandle<()>,
   state_updater: watch::Sender<State>,
}


use tokio::sync::mpsc;


#[derive(Debug, Clone)]
pub struct TopLevelMetric
{
   pub value: f64,
   pub timestamp: i64,
}

impl TryFrom<Scrape> for TopLevelMetric
{
   type Error = APIError;
   fn try_from(scrape: Scrape) -> Result<Self, Self::Error>
   {
      let cpu_metric = scrape
         .metrics
         .into_iter()
         .find(|metric| metric.name == "container_cpu_usage_seconds_total")
         .ok_or(APIError::CPUMetricNotFound)?;

      let top_level_metric = cpu_metric
         .samples
         .into_iter()
         .find(|sample| {
            sample
               .labels
               .iter()
               .find(|label| label.key == "id" && label.value == "/")
               .is_some()
         })
         .ok_or(APIError::NodeTopLevelContainerMetricNotFound)?;

      let value = top_level_metric.value;
      let Value {
         value, timestamp, ..
      } = value;
      let value = value.as_f64();
      let timestamp = timestamp.ok_or(APIError::NodeTopLevelContainerMetricNoTimeStamp)?;

      Ok(Self { value, timestamp })
   }
}


async fn query_node_c_advisor(
   client: &KubeClient,
   pod: &Pod,
) -> Result<NodeMetric, APIError>
{
   let response = client.proxy.pod(&pod, "metrics").await?;
   let string = response.text().await?;
   let scrape = Scrape::parse(&string)?;
   let top_level_metric: TopLevelMetric = scrape.try_into()?;
   let node_metric = NodeMetric {
      uid: pod.uid.clone().into(),
      metric: top_level_metric,
   };

   Ok(node_metric)
}

impl QueryTask
{
   pub fn new(
      pod: &Pod,
      metric_sender: mpsc::Sender<NodeMetric>,
      client: &KubeClient,
   ) -> Self
   {
      println!("querier task created for {} with init state: {}", pod.name, pod.status);
      let pod = pod.clone();
      let client = client.clone();
      let init_state = if pod.status { State::Running } else { State::Paused };
      let (state_updater, mut state_reader) = watch::channel(init_state);
      let task = async move || {
         let pod = pod;

         loop {
            match *state_reader.borrow_and_update() {
               State::Killed => return println!("querier killed"),
               State::Running => break,
               State::Paused => println!("initial state is paused awaiting state change..."),
            };

            match state_reader.changed().await {
               Ok(_) => {
                  println!("state has changed after intial pause. re-evaluating");
               },
               Err(e) => {
                  println!("Error occured reading state after initial pause. terminating with error:\n{e}");
                  return;
               },
            };
         };

         loop {
            match state_reader.has_changed() {
               Ok(true) => loop {
                  println!();
                  println!("state has changed");
                  println!();
                  match *state_reader.borrow_and_update() {
                     State::Running => (),
                     State::Killed => return println!("querier killed"),
                     State::Paused => continue,
                  };
               },

               Ok(false) => (),
               Err(e) => {
                  println!("Error from node querying 2:\n{e:?}");
                  return;
               }
            };

            let metric = match query_node_c_advisor(&client, &pod).await {
               Ok(v) => v,
               Err(e) => {
                  println!("Error from node querying 3:\n{e:?}");
                  tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                  continue;
               }
            };

            match metric_sender.send(metric).await {
               Ok(_) => (),
               Err(e) => {
                  println!("Error from node querying 4:\n{e:?}");
                  return;
               }
            };
         };
      };

      let handle = tokio::spawn(task());


      Self { handle, state_updater }
   }


   pub fn pause(&self) {
      match self.state_updater.send(State::Paused) {
         Err(e) => println!("Error from node querying 5:\n{e:?}"),
         _ => (),
      };
   }

   pub fn resume(&self) {
      match self.state_updater.send(State::Running) {
         Err(e) => println!("Error from node querying 6:\n{e:?}"),
         _ => (),
      };
   }

   pub async fn kill(self) {
      match self.state_updater.send(State::Killed) {
         Err(e) => println!("Error from node querying 7:\n{e:?}"),
         _ => (),
      };

      match self.handle.await {
         Err(e) => println!("Error from node querying 8:\n{e:?}"),
         _ => (),
      };
   }


}

