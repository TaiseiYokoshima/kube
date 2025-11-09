use std::time::Duration;
use tokio::sync::mpsc;

use k8s_openapi::{
   api::apps::v1::ReplicaSet,
   apimachinery::pkg::apis::meta::v1::WatchEvent
};

use bytes::{Bytes, BytesMut};
use tokio::time::sleep;

use super::super::{
   ClientError, ReplicaSets, client::Base, replicasets::PodsCount, replicasets::TemplateHash,
   Target,
};

use super::watcher::Watcher;


fn parse_replica_set(set: ReplicaSet) -> Result<(TemplateHash, PodsCount), ClientError> {
   let ReplicaSet {
      metadata, status, ..
   } = set;

   let hash = metadata
      .labels
      .ok_or(ClientError::Json("no labels in replica set's metadata"))?
      .remove("pod-template-hash")
      .ok_or(ClientError::Json(
         "no hash in replica set's metadata labels ",
      ))?
      .into();

   let pods_count = status
      .ok_or(ClientError::Json("no status in replica set"))?
      .replicas as PodsCount;

   Ok((hash, pods_count))
}

impl TryFrom<WatchEvent<ReplicaSet>> for ReplicaSetEvent {
   type Error = ClientError;

   fn try_from(value: WatchEvent<ReplicaSet>) -> Result<Self, Self::Error> {
      let value = match value {
         WatchEvent::Added(set) => {
            let (hash, count) = parse_replica_set(set)?;
            ReplicaSetEvent::Created(hash, count)
         }
         WatchEvent::Modified(set) => {
            let (hash, count) = parse_replica_set(set)?;
            ReplicaSetEvent::Modified(hash, count)
         }
         WatchEvent::Deleted(set) => {
            let (hash, count) = parse_replica_set(set)?;
            ReplicaSetEvent::Deleted(hash, count)
         }

         _ => ReplicaSetEvent::__,
      };

      Ok(value)
   }
}




async fn reacquire_and_diff(
   client: &Base,
   targets: &Vec<Target>,
   old_sets: &mut ReplicaSets,
   sender: &mpsc::Sender<Result<ReplicaSetEvent, ClientError>>,
) 
   -> Result<(), ClientError>
{
   use super::super::get_replicasets::get_replica_sets;

   let duration = Duration::from_secs(10);
   println!("about to restablish and diff ({} secs)", duration.as_secs());
   sleep(duration).await;



   let mut new_sets = get_replica_sets(client, targets).await?;

   let old_maps = &mut old_sets.hashes;


   for (hash, count) in new_sets.hashes.iter() {

      let event = match old_maps.remove(hash) {
         Some(old) => {
            if old == *count {
               continue;
            };
            ReplicaSetEvent::Modified(hash.clone(), *count)
         },
         None => ReplicaSetEvent::Created(hash.clone(), *count),
      };

      match sender.send(Ok(event)).await {
         Ok(()) => (),
         Err(_) => return Err(ClientError::SenderError("watch api on replica sets")),
      };
   };

   for (hash, count) in old_maps {
      let event = ReplicaSetEvent::Deleted(hash.clone(), *count);
      match sender.send(Ok(event)).await {
         Ok(()) => (),
         Err(_) => return Err(ClientError::SenderError("watch api on replica sets")),
      };
   };


   std::mem::swap(old_sets, &mut new_sets);

   Ok(())
}


fn handle_event(event: &ReplicaSetEvent, replica_sets: &mut ReplicaSets) {
   let maps = &mut replica_sets.hashes;
   match event {
      ReplicaSetEvent::Deleted(hash, _) => assert!(maps.remove(hash).is_some()),
      ReplicaSetEvent::Modified(hash, count) => *maps.get_mut(hash).unwrap() = *count,
      ReplicaSetEvent::Created(hash, count) => assert!(maps.insert(hash.clone(), *count).is_none()),
      ReplicaSetEvent::__ => (),
   }
}

async fn query_task(
   client: Base,
   mut replica_sets: ReplicaSets,
   sender: mpsc::Sender<Result<ReplicaSetEvent, ClientError>>,
   targets: Vec<Target>,
   timeout: Duration,
) -> () {

   use super::watcher::get_stream;
   use super::watcher::get_event;

   let event_builder = &mut BytesMut::new();

   // this loop is establishing watch api connection stream
   let error = 'outer: loop {
      let url = format!(
         "/apis/apps/v1/replicasets?watch=true&resourceVersion={}&timeoutSeconds={}",
         &replica_sets.version,
         timeout.as_secs(),
      );

      let mut stream = match get_stream(&client, &url).await {
         Err(e) => break 'outer e,
         Ok(stream) => stream,
      };

      // this loop is getting events from stream
      loop {
         let event: ReplicaSetEvent = match get_event(event_builder, &mut stream).await {
            Err(e) => {

               if let ClientError::WatchTimedOut = e {
                  match reacquire_and_diff(&client, &targets, &mut replica_sets, &sender).await {
                     Ok(_) => continue 'outer,
                     Err(error) => break 'outer error,
                  };
               };

               break 'outer e;
            },

            Ok(value) => value,
         };


         handle_event(&event, &mut replica_sets);

         match sender.send(Ok(event)).await {
            Err(_) => break 'outer ClientError::SenderError("watch api on replica sets"),
            _ => (),
         };
      }
   };

   if let ClientError::SenderError(_) = error {
      println!("{error}");
      return;
   };


   let result = sender.send(Err(error)).await;

   if result.is_err() {
      println!("{result:?}");
   };
}

#[derive(Debug)]
pub enum ReplicaSetEvent {
   Modified(TemplateHash, PodsCount),
   Created(TemplateHash, PodsCount),
   Deleted(TemplateHash, PodsCount),
   __,
}

impl Watcher<ReplicaSet, ReplicaSetEvent> {
   pub async fn next_event(&mut self) -> Result<ReplicaSetEvent, ClientError> {
      match self.receiver.recv().await {
         None => Err(ClientError::ReceiverError("watch api on replica sets")),
         Some(value) => value
      }
   }

   pub fn new(client: Base, replica_sets: ReplicaSets, targets: Vec<Target>, timeout: Duration) -> Self {
      let (sender, receiver) = mpsc::channel(100);
      let task = query_task(client, replica_sets, sender, targets, timeout);
      let handle = tokio::spawn(task);
      let resource = std::marker::PhantomData;

      Self {
         handle,
         receiver,
         resource
      }
   }
}
