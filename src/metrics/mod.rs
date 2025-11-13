use tokio::sync::watch;
use crate::{client::{KubeClient, Pod}};


use prom_text_format_parser::{Sample, Scrape};
use prom_text_format_parser::Metric;


fn get_cpu_metrics(data: Scrape) -> Option<Metric> {
   data.metrics.into_iter().find(|metric| metric.name == "container_cpu_usage_seconds_total")
}

struct CAdvisor {
   uids: Vec<Box<str>>,
   values: Vec<f64>,
}



fn get_uid_from_sample(sample: &Sample) -> Box<str> {
   


}

impl From<Metric> for CAdvisor {
   fn from(value: Metric) -> Self {
      let mut uids = Vec::new();
      let mut values = Vec::new();


      for sample in value.samples {
         let Sample {
            value,
            labels,
         } = sample;

         
      };
       
   }
}


pub async fn query_cadvisor(client: &KubeClient, pod: &Pod) {
   let response = client.proxy.pod(&pod, "metrics").await;
   let string = response.unwrap().text().await.unwrap();
   let scrape = Scrape::parse(&string).unwrap();
   let metric = get_cpu_metrics(scrape).unwrap();



"container_label_io_kubernetes_pod_name"
"container_label_io_kubernetes_pod_namespace"




   for metric in scrape.metrics {
      if metric.name != "container_cpu_usage_seconds_total" { continue; };

      for sample in metric.samples {
         sample.labels.iter().for_each(|label| { label_set.insert(label.key.clone()); });
      };


      let set = label_set.into_iter().collect::<Vec<String>>();

      for label in set {
         println!("{:?}", label);
      };

      return;
   };


}



pub async fn cadvisor_querier(client: KubeClient, index: watch::Receiver<usize>, resumed: watch::Receiver<bool>, pod: Pod) {



}



