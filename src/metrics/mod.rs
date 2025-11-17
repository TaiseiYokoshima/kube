use crate::client::{KubeClient, Pod};
use prom_text_format_parser::{Sample, Scrape, Metric};
mod total;
mod controller;

fn get_cpu_metrics(data: Scrape) -> Option<Metric> {
   data
      .metrics
      .into_iter()
      .find(|metric| metric.name == "container_cpu_usage_seconds_total")
}

#[derive(Debug, Clone)]
struct KubeMetaData {
   pub pod_name: String,
   pub namespace: String,
   pub pod_uid: String,
}

#[derive(Debug, Clone)]
struct ContainerMetric {
   pub id: String,
   pub kube_meta: Option<KubeMetaData>,
   pub value: f64,
   pub timestamp: i64,
}

impl std::fmt::Display for ContainerMetric {
   fn fmt(
      &self,
      f: &mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      if self.kube_meta.is_none() {
         return write!(f, "id: `{}`", self.id);
      };

      let KubeMetaData {
         pod_name,
         namespace,
         ..
      } = self.kube_meta.as_ref().unwrap();
      write!(f, "{}/{}", namespace, pod_name)
   }
}

impl TryFrom<Sample> for ContainerMetric {
   type Error = ();
   fn try_from(sample: Sample) -> Result<Self, Self::Error> {
      let Sample { labels, value } = sample;

      let time = value.timestamp.ok_or(())?;
      let value = value.value.as_f64();

      let pod_name = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_name")
         .map(|x| x.value.clone());

      let namespace = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_namespace")
         .map(|x| x.value.clone());

      let pod_uid = labels
         .iter()
         .find(|label| label.key == "container_label_io_kubernetes_pod_uid")
         .map(|x| x.value.clone());

      let kube_meta = match (pod_uid, namespace, pod_name) {
         (Some(pod_uid), Some(namespace), Some(pod_name)) => Some(KubeMetaData {
            pod_name,
            pod_uid,
            namespace,
         }),
         _ => None,
      };

      let id = labels
         .iter()
         .find(|label| label.key == "id")
         .ok_or(())?
         .value
         .clone();

      Ok(ContainerMetric {
         id,
         kube_meta,
         value,
         timestamp: time,
      })
   }
}

pub async fn query_cadvisor(
   client: &KubeClient,
   pod: &Pod,
) {
   let mut total = total::TotalMetric::new();
   // let mut previous: Option<ContainerMetric> = None;

   loop {
      let response = client.proxy.pod(&pod, "metrics").await;
      let string = response.unwrap().text().await.unwrap();
      let scrape = Scrape::parse(&string).unwrap();
      let metric = get_cpu_metrics(scrape).unwrap();

      let container = match metric
         .samples
         .into_iter()
         .map(|x| x.try_into())
         .filter_map(|x: Result<ContainerMetric, ()>| x.ok())
         .find(|x| x.id == "/") {
            Some(x) => x,
            None => continue,
         };

      // println!("found container: {} @ {}", container.id, container.timestamp);
      let (x, y) = total.next(container.timestamp, container.value);
      println!("{y}% @ {x} (unix epoch)");
   }
}
