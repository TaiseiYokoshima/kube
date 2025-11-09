use std::{collections::HashSet, time::Duration};

use super::{APIError, Base, CAdvisorDaemonSetMetadata, CAdvisorPods, errors, response_into_error, Pod, get_daemon_set_pods};

use futures::{StreamExt, future::FutureExt};
use futures_core::stream::Stream;

use bytes::Bytes;
use k8s_openapi::{api::core::v1::Pod as JsonPod, apimachinery::pkg::apis::meta::v1::{ObjectMeta, WatchEvent}};


#[derive(Debug)]
pub enum EventKind
{
   Created,
   Deleted,
   Paused,
   Resumed,
}

#[derive(Debug)]
pub struct DaemonSetEvent
{
   pod: Pod,
   kind: EventKind,
}


impl std::fmt::Display for DaemonSetEvent {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "Pod {}: ", self.pod.name)?;
      match self.kind {
         EventKind::Created => {
            let running = if self.pod.status { "running" } else { "paused" };
            writeln!(f, "created - {}", running)
         },
         EventKind::Deleted => writeln!(f, "deleted"),
         EventKind::Paused => writeln!(f, "paused"),
         EventKind::Resumed => writeln!(f, "resumed"),
      }
   }
}

use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct Watcher<Resource, Data>
{
   terminator: oneshot::Sender<()>,
   receiver: mpsc::Receiver<Result<Data, WatcherError>>,
   handle: tokio::task::JoinHandle<()>,
   phantom: std::marker::PhantomData<Resource>,
}

#[derive(Debug)]
pub enum WatcherErrorKind
{
   StreamRetrieveError,
   ReconilationError,
   StreamNextError,
   Termination,
}

#[derive(Debug)]
pub struct WatcherError
{
   cause: WatcherErrorKind,
   error: APIError,
}

impl WatcherError
{
   fn recon(error: APIError) -> Self
   {
      Self {
         cause: WatcherErrorKind::ReconilationError,
         error,
      }
   }

   fn stream(error: APIError) -> Self
   {
      Self {
         cause: WatcherErrorKind::StreamRetrieveError,
         error,
      }
   }

   fn next(error: APIError) -> Self
   {
      Self {
         cause: WatcherErrorKind::StreamNextError,
         error,
      }
   }

   fn terminate() -> Self
   {
      Self {
         cause: WatcherErrorKind::Termination,
         error: APIError::WatcherTermination,
      }
   }
}

impl Watcher<CAdvisorDaemonSetMetadata, DaemonSetEvent>
{
   pub fn new(
      client: Base,
      daemon_set: CAdvisorDaemonSetMetadata,
      state: CAdvisorPods,
      duration: Duration,
   ) -> Self
   {

      let (terminator, kill_signal) = oneshot::channel::<()>();
      let (sender, receiver) = mpsc::channel(100);
      let handle = tokio::spawn(watch_daemon_set_pods(client, state, daemon_set, duration, sender, kill_signal));

      Self {
         terminator,
         receiver,
         handle,
         phantom: std::marker::PhantomData,
      }
   }

   pub async fn next(&mut self) -> Result<DaemonSetEvent, WatcherError> {
      match self.receiver.recv().await {
         Some(Ok(v)) => Ok(v),
         Some(Err(e)) => Err(e),
         None => Err(WatcherError::next(APIError::ChannelSenderDropped)),
      }
   }

   pub async fn kill(self) -> Result<(), WatcherError> {
      match self.terminator.send(()) {
         Ok(_) => Ok(()),
         Err(_) => Err(WatcherError::terminate()),
      }
   }
}


pub async fn reconcile(
   client: &Base,
   daemon_set: &CAdvisorDaemonSetMetadata,
   old_state: &mut CAdvisorPods,
) -> Result<(CAdvisorPods, Vec<DaemonSetEvent>), WatcherError>
{
   println!("reconcilation running");


   let mut events: Vec<DaemonSetEvent> = vec![];

   let new_state = get_daemon_set_pods(client, daemon_set)
      .await
      .map_err(|e| WatcherError::recon(e))?;

   let uids: HashSet<_> = new_state.uids().chain(old_state.uids()).collect();

   for uid in uids {
      let event = match (new_state.get_by_uid(uid), old_state.get_by_uid(uid)) {
         (Some(new), Some(old)) => {
            if new.status == old.status {
               continue;
            };

            let kind = if new.status {
               EventKind::Resumed
            } else {
               EventKind::Paused
            };

            DaemonSetEvent {
               pod: new.clone(),
               kind,
            }
         }
         (None, Some(old)) => DaemonSetEvent {
            pod: old.clone(),
            kind: EventKind::Deleted,
         },
         (Some(new), None) => DaemonSetEvent {
            pod: new.clone(),
            kind: EventKind::Created,
         },
         _ => unreachable!(),
      };

      events.push(event);
   }

   Ok((new_state, events))
}


async fn get_stream(
   client: &Base,
   endpoint: &str,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, WatcherError>
{
   let mut retry_allowance = 5;

   let stream = loop {
      // first make the request
      let response = client
         .get(endpoint)
         .send()
         .await
         .map_err(|e| WatcherError::stream(e.into()));
      let response = match response {
         Err(e) => {
            if retry_allowance == 0 {
               return Err(e);
            };

            retry_allowance -= 1;
            continue;
         }
         Ok(response) => response,
      };

      // check if the response was error
      let response = response_into_error(response)
         .await
         .map_err(|e| WatcherError::stream(e.into()));
      let error = match response {
         Ok(response) => break response.bytes_stream(),
         Err(e) => e,
      };

      if retry_allowance == 0 {
         return Err(error);
      };

      retry_allowance -= 1;
   };

   Ok(stream)
}



fn parse_pod_object(pod: JsonPod) -> Result<Pod, APIError> {
   let JsonPod {
      metadata, status, ..
   } = pod;


   let ObjectMeta {
      uid, name, ..
   } = metadata;

   let uid = match uid {
      Some(uid) => uid,
      None => return Err(errors::UID),
   };

   let name = match name {
      Some(name) => name,
      None => return Err(errors::NAME),
   };



   let status = match status
      .ok_or(errors::STATUS)
      .and_then( |x| x.conditions.ok_or(errors::CONDITION))
      .and_then(|x|
          x.iter()
         .find_map(|status| 
            if status.type_ != "Ready" {
               None
            } else {
               if status.status == "True" {
                  Some(true)
               } else {
                  Some(false)
               }
            }
         ).ok_or(errors::READY)
      ) {
         Ok(status) => status,
         Err(e) => {
            println!("got {e:?} in parse but fell back to false");
            false
         },

      };


   Ok(Pod::new(uid.into(), name.into(), status))
}
       


async fn get_events(
   stream: &mut (impl Stream<Item = Result<Bytes, reqwest::Error>> + Unpin),
   event_builder: &mut Vec<u8>,
) -> Result<Option<Vec<WatchEvent<JsonPod>>>, WatcherError>
{
   let mut events = vec![];

   let bytes = match stream.next().await {
      None => return Ok(None),
      Some(Err(e)) => return Err(WatcherError::next(e.into())),
      Some(Ok(bytes)) => bytes,
   };

   for byte in bytes {
      if byte == b'\n' {
         let bytes = std::mem::take(event_builder);
         let event: WatchEvent<JsonPod> = serde_json::from_slice(&bytes).map_err(|e| WatcherError::next(e.into()))?;
         event_builder.clear();
         events.push(event);
         continue;
      };

      event_builder.push(byte);
   }

   Ok(Some(events))
}


pub async fn watch_daemon_set_pods(
   client: Base,
   mut state: CAdvisorPods,
   daemon_set: CAdvisorDaemonSetMetadata,
   duration: Duration,
   sender: mpsc::Sender<Result<DaemonSetEvent, WatcherError>>,
   kill_signal: oneshot::Receiver<()>,
)
{
   let seconds = duration.as_secs();
   let kill_signal = kill_signal.shared();

   let mut event_builder = vec![];

   'reconnection: loop {
      let endpoint = format!(
         "/api/v1/namespaces/{}/pods?labelSelector={}={}&watch=true&resourceVersion={}&timeoutSeconds={}",
         daemon_set.namespace,
         daemon_set.key,
         daemon_set.value,
         state.version,
         seconds,
      );


      let stream = tokio::select! {
            _ = kill_signal.clone() => return,
            stream = get_stream(&client, &endpoint) => stream,
      };

      let mut stream = match stream {
         Ok(stream) => stream,
         Err(e) => {
            let _ = sender.send(Err(e)).await;
            return;
         },
      };

      
      loop {

         let events = tokio::select! {
            _ = kill_signal.clone() => return,
            events = get_events(&mut stream, &mut event_builder) => events,
         };

         let events = match events {
            Err(e) => {
               let _ = sender.send(Err(e)).await;
               return;
            },
            Ok(None) => {

               let (new_state, events) = match reconcile(&client, &daemon_set, &mut state).await {
                  Err(e) => {
                     let _ = sender.send(Err(e)).await;
                     return;
                  },
                  Ok(output) => output,
               };

               state = new_state;

               for event in events {
                  println!("event sending: reconcilation");
                  if sender.send(Ok(event)).await.is_err() {
                     return
                  };
               };

               continue 'reconnection;
            },
            Ok(Some(events)) => events,
         };

         
         for event in events {
            let event = match event {
               WatchEvent::Added(pod) => {
                  let pod = match parse_pod_object(pod) {
                     Ok(v) => v,
                     Err(e) => {
                        let _ = sender.send(Err(WatcherError::next(e.into()))).await;
                        return;
                     }
                  };

                  state.insert(pod.clone());

                  DaemonSetEvent {
                     pod: pod,
                     kind: EventKind::Created,
                  }
               },

               WatchEvent::Deleted(pod) => {
                  let pod = match parse_pod_object(pod) {
                     Ok(v) => v,
                     Err(e) => {
                        let _ = sender.send(Err(WatcherError::next(e.into()))).await;
                        return;
                     },
                  };

                  state.remove(&pod);

                  DaemonSetEvent {
                     pod: pod,
                     kind: EventKind::Deleted
                  }
               },

               WatchEvent::Modified(pod) => {
                  let new = match parse_pod_object(pod) {
                     Ok(v) => v,
                     Err(e) => {
                        let _ = sender.send(Err(WatcherError::next(e.into()))).await;
                        return;
                     },
                  };

                  let old = match state.get_mut(&new) {
                     Some(old) => old,
                     _ => unreachable!(),
                  };

                  if old.status == new.status {
                     continue;
                  };

                  old.status = new.status;
                      
                  let kind = if new.status {
                     EventKind::Resumed
                  } else {
                     EventKind::Paused
                  };


                  DaemonSetEvent {
                     pod: new,
                     kind,
                  }

               },
               _ => continue,
            };


            if sender.send(Ok(event)).await.is_err() {
               return;
            };

         };



      };

   };


}
