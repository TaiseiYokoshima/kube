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

   let duration = std::time::Duration::from_secs(10);


   // let mut watcher = client.watch.daemon_set_pods (daemon_set_meta, daemon_set_state, duration);
   // loop {
   //     let event = watcher.next().await.unwrap();
   //     println!("event: {:?} from {}", event.kind, event.pod.name);
   // };

   let metric = metrics::MetricCollector::new(client, daemon_set_meta, daemon_set_state);

   tokio::signal::ctrl_c().await.unwrap();
   let result = metric.kill().await;
   println!("got the scrape result");
}
