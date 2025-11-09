use std::time::Duration;
use bytes::BytesMut;
use k8s_openapi::{List, api::core::v1::Node, apimachinery::pkg::apis::meta::v1::WatchEvent};

use tokio::sync::mpsc;

use super::super::{Base, ClientError};
use super::Watcher;
use super::super::Nodes;


#[derive(Debug)]
pub enum NodeEvent {
   Modified(Box<str>, bool),
   Created(Box<str>, bool),
   Deleted(Box<str>, bool),
   __,
}

fn parse_node(node: Node) -> Result<(Box<str>, bool), ClientError> {
   let Node {
      metadata, status, ..
   } = node;


   let name = metadata.name.ok_or(ClientError::Json("no name in node's metadata"))?;

   let condition = status
      .ok_or(ClientError::Json("no status in node"))?
      .conditions
      .ok_or(ClientError::Json("no conditions in node's status"))?;

   let ready_status = condition
      .into_iter()
      .find(|condition| condition.type_ == "Ready")
      .ok_or(ClientError::Json(
         "no ready condition in node's status conditions",
      ))?
      .status;

   let status = if ready_status == "True" { true } else { false };

   Ok((name.into(), status))
}

impl TryFrom<WatchEvent<Node>> for NodeEvent {
   type Error = ClientError;

   fn try_from(value: WatchEvent<Node>) -> Result<Self, Self::Error> {
      let value = match value {
         WatchEvent::Added(node) => {
            let (name, status) = parse_node(node)?;
            NodeEvent::Created(name, status)
         }
         WatchEvent::Modified(node) => {
            let (name, status) = parse_node(node)?;
            NodeEvent::Modified(name, status)
         }
         WatchEvent::Deleted(node) => {
            let (name, status) = parse_node(node)?;
            NodeEvent::Deleted(name, status)
         }

         _ => NodeEvent::__,
      };

      Ok(value)
   }
}

async fn reacquire_and_diff(client: &Base, nodes: &mut Nodes,  sender: &mpsc::Sender<Result<NodeEvent, ClientError>>) -> Result<(), ClientError> {
   use super::super::get_nodes::get_nodes;
   let nodes = get_nodes(client).await?;







   Ok(())
}


fn handle_event(event: NodeEvent, nodes: &mut Nodes) -> Option<NodeEvent> {
   match &event {
      NodeEvent::Modified(name, status) => {
         let node_pos = nodes.names.iter().position(|node| node.as_ref() == name.as_ref()).unwrap();
         let old_status = nodes.statuses.get_mut(node_pos).unwrap();

         if *status == *old_status {
            return None;
         };

         *old_status = *status;
      },

      NodeEvent::Deleted(name, _) => {
         let node_pos = nodes.names.iter().position(|node| node.as_ref() == name.as_ref()).unwrap();
         nodes.names.remove(node_pos);
         nodes.statuses.remove(node_pos);
      },

      NodeEvent::Created(name, status) => {
         nodes.names.push(name.clone());
         nodes.statuses.push(*status);
      },

      _ => return None,
   };

   Some(event)
}


async fn query_task(client: Base, mut nodes: Nodes, sender: mpsc::Sender<Result<NodeEvent, ClientError>>, timeout: Duration) {
   use super::watcher::get_stream;
   use super::watcher::get_event;


   let mut event_builder = BytesMut::new();

   let error = 'outer: loop {
      let url = format!("/api/v1/watch/nodes?resourceVersion={}&timeoutSeconds={}", nodes.version, timeout.as_secs());
      let mut stream = match get_stream(&client, &url).await {
         Ok(stream) => stream,
         Err(error) => break 'outer error,
      };
      

      'inner: loop {
         let event: NodeEvent = match get_event(&mut event_builder, &mut stream).await {
            Err(error) => { 
               if let ClientError::WatchTimedOut = error {
                  match reacquire_and_diff(&client, &mut nodes, &sender).await {
                     Err(e) => break 'outer e,
                     _ => continue 'outer,
                  };

               };

               break 'outer error;
            },
            Ok(event) => event,
         };

         let event = match handle_event(event, &mut nodes) {
            None => continue 'inner,
            Some(event) => event,
         };

         match sender.send(Ok(event)).await {
            Err(_) => break 'outer ClientError::SenderError("watch api on replica sets"),
            _ => (),
         };

      };


      

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

impl Watcher<Node, NodeEvent> {
   pub async fn next_event(&mut self) -> Result<NodeEvent, ClientError> {
      match self.receiver.recv().await {
         None => Err(ClientError::ReceiverError("watch api on replica sets")),
         Some(value) => value,
      }
   }

   pub fn new(
      client: Base,
      nodes: Nodes,
      timeout: Duration,
   ) -> Self {
      let (sender, receiver) = mpsc::channel(100);
      let task = query_task(client, nodes, sender, timeout);
      let handle = tokio::spawn(task);
      let resource = std::marker::PhantomData;

      Self {
         handle,
         receiver,
         resource,
      }
   }
}
