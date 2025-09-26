use kube::client::KubeClient;

use std::rc::Rc;
use std::time::Duration;

use kube::client::Base;
use kube::client::ClientError;

use k8s_openapi::api::apps::v1::ReplicaSet;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::WatchEvent;

use futures::stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};




async fn watch_nodes(client: &Rc<Base>, version: &Box<str>) -> Result<(), ClientError> {
   let mut lines = {
      let url = format!(
         "/apis/apps/v1/replicasets?watch=true&resourceVersion={}&timeoutSeconds=1000000000",
         version
      );
      let stream = client
         .get(url)
         .send()
         .await?
         .error_for_status()?
         .bytes_stream()
         .map(|b| b.map_err(std::io::Error::other));

      let reader = tokio_util::io::StreamReader::new(stream);
      FramedRead::new(reader, LinesCodec::new())
   };

   while let Some(line) = lines.next().await {
      match &line {
         Err(e) => println!("Error:\n{}\n{:?}", e, e),
         _ => (),
      };

      let line = line.unwrap();
      let event: WatchEvent<ReplicaSet> = serde_json::from_str(&line).unwrap();

      match event {
         WatchEvent::Added(node) => {
            println!("added");
         }

         WatchEvent::Deleted(node) => {
            println!("deleted");
         }

         WatchEvent::Modified(node) => {
            println!("{:?}", node.metadata.resource_version);
            println!("modified");
         }

         other => {
            println!("other: {:?}", other);
         }
      }
   }

   println!("no more events");

   Ok(())
}


fn test() {

   let mut first = vec![1, 2, 3];
   let mut second = vec![4, 5, 6];

   println!("first: {first:?}");
   println!("second: {second:?}");

   std::mem::swap(&mut first, &mut second);

   println!("first: {first:?}");
   println!("second: {second:?}");
      
}

#[tokio::main]
async fn main() {





   use kube::client::TargetInput;

   let client = KubeClient::new().unwrap();

   let t1 = TargetInput::new("default", "stress-test", vec!["stress"]);

   let inputs = vec![t1];

   let results = client.validate.targets(inputs).await;

   let targets = match results {
      Ok(targets) => targets,
      Err(error) => return println!("{error}"),
   };

   let nodes = client.get.nodes().await.unwrap();
   let replica_sets = client.get.replica_sets(&targets).await.unwrap();
   let pods = client.get.pods(&replica_sets).await.unwrap();

   let timeout = Duration::from_secs(10);
   let mut watcher = client.watch.replica_sets(targets, replica_sets, timeout);

   loop {
      let event = watcher.next_event().await.unwrap();
      println!("event: {:?}", event);

   };

}
