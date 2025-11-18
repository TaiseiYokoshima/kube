use kube::client::CAdvisorDaemonSetMetadata;
use kube::client::KubeClient;

use kube::metrics;

#[tokio::main]
async fn main()
{


   let client = KubeClient::new().unwrap();

   let namespace = "kube-system".into();
   let key = "k8s-app".into();
   let value = "cadvisor".into();

   let daemon_set_meta = CAdvisorDaemonSetMetadata {
      key,
      value,
      namespace,
   };

   let daemon_set_state = client.get.daemon_set_pods(&daemon_set_meta).await.unwrap();
   let metric = metrics::MetricCollector::new(client.clone(), daemon_set_meta, daemon_set_state);

   match tokio::signal::ctrl_c().await {
      Ok(_) => (),
      Err(e) => println!("Error failed to intercept kill signal: {e}"),
   };  
   
   let result = metric.kill().await;
   println!("got the scrape result");
}
