use kube::client::CAdvisorDaemonSetMetadata;
use kube::client::KubeClient;

#[tokio::main]
async fn main()
{
   let client = KubeClient::new().unwrap();

   let namespace = "kube-system".into();
   let key = "k8s-app".into();
   let value = "cadvisor".into();

   let set = CAdvisorDaemonSetMetadata {
      key,
      value,
      namespace,
   };

   let pods = client.get.daemon_set_pods(&set).await.unwrap();

   println!("got pods: {pods}");

   let duration = std::time::Duration::from_secs(20);

   let mut watcher = client.watch.daemon_set_pods(set, pods, duration);

   loop {
      let event = watcher.next().await.unwrap();
      println!("{event}");
   };
}
