use crate::client::{KubeClient, Pod};
use tokio::sync::watch;

use prom_text_format_parser::Metric;
use prom_text_format_parser::{Sample, Scrape};

fn get_cpu_metrics(data: Scrape) -> Option<Metric>
{
   data
      .metrics
      .into_iter()
      .find(|metric| metric.name == "container_cpu_usage_seconds_total")
}

#[derive(Debug, Clone)]
struct ContainerMetric
{
   pub uid: String,
   pub namespace: String,
   pub name: String,
   pub value: f64,
   pub time: i64,
}

use std::collections::HashMap;

#[repr(transparent)]
#[derive(Debug)]
struct MetricState(HashMap<String, ContainerMetric>);

impl MetricState
{
   pub fn new() -> Self
   {
      MetricState(HashMap::new())
   }

   pub fn next(
      &mut self,
      containers: Vec<ContainerMetric>,
   ) -> f64
   {
      let mut sum = 0.0;

      for container in containers {
         let previous = match self.0.get_mut(&container.uid) {
            Some(x) => x,
            _ => {
               self.0.insert(container.uid.clone(), container);
               continue;
            }
         };



         let value_d = container.value - previous.value;

         let time_d = {
            let mut time_delta = container.time - previous.time;
            if time_delta.is_negative() {
               time_delta = std::i64::MAX - previous.time + container.time;
            };

            time_delta as f64 / 1000.0
         };


         if time_d < 0.0 {
            println!("time delta was negative");
         };

         let percentage = value_d / time_d;
         sum += percentage;
         let _ = std::mem::replace(previous, container);
      }

      sum
   }
}

impl TryFrom<Sample> for ContainerMetric
{
   type Error = ();
   fn try_from(sample: Sample) -> Result<Self, Self::Error>
   {
      let Sample { labels, value } = sample;

      let time = value.timestamp.ok_or(())?;
      let value = value.value.as_f64();
      let name = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_name")
         .ok_or(())?
         .value
         .clone();
      let namespace = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_namespace")
         .ok_or(())?
         .value
         .clone();
      let uid = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_uid")
         .ok_or(())?
         .value
         .clone();

      Ok(ContainerMetric {
         uid,
         namespace,
         name,
         value,
         time,
      })
   }
}


fn get_pod_measurement(metric: Metric)
{
   for sample in metric.samples {
      let container_metric = sample.try_into().unwrap();
      let ContainerMetric {
         name,
         time,
         namespace,
         value,
         ..
      } = &container_metric;

      println!("{namespace}/{name} | {value} @ {time}");
   }
}

//"container_label_io_kubernetes_pod_name"
//"container_label_io_kubernetes_pod_namespace"


pub async fn query_cadvisor(
   client: &KubeClient,
   pod: &Pod,
)
{
   let mut state = MetricState::new();

   let Pod {
      name,
      ..
   } = &pod;

   println!("now reading total cpu usage from {name}:");
   loop {
      let response = client.proxy.pod(&pod, "metrics").await;
      let string = response.unwrap().text().await.unwrap();
      let scrape = Scrape::parse(&string).unwrap();
      let metric = get_cpu_metrics(scrape).unwrap();
      let containers: Vec<ContainerMetric> = metric.samples.into_iter().map(|x| x.try_into()).filter_map(|x: Result<ContainerMetric, ()>| x.ok()).collect();
      let percentage = state.next(containers);
      println!("{percentage}%");
   };

}

pub async fn cadvisor_querier(
   client: KubeClient,
   index: watch::Receiver<usize>,
   resumed: watch::Receiver<bool>,
   pod: Pod,
)
{
}
