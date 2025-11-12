use tokio::sync::watch;
use crate::{client::{KubeClient, Pod}, metrics};


use prom_text_format_parser::Scrape;

pub async fn query_cadvisor(client: &KubeClient, pod: &Pod) {
   let response = client.proxy.pod(&pod, "metrics").await;
   let string = response.unwrap().text().await.unwrap();
   let scrape = Scrape::parse(&string).unwrap();



   let mut label_set = std::collections::HashSet::new();
   

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



