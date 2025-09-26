use std::marker::PhantomData;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::WatchEvent;

use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use bytes::{Bytes, BytesMut};

use super::super::ClientError;
use super::super::Base;


pub struct Watcher<Resource, Event> 
where
   Resource: serde::de::DeserializeOwned,
   Event: TryFrom<WatchEvent<Resource>, Error = ClientError>,
{
   pub handle: JoinHandle<()>,
   pub receiver: Receiver<Result<Event, ClientError>>,
   pub resource: PhantomData<Resource>,
}

pub async fn get_event<Resource, Event>(
   event_builder: &mut BytesMut,
   stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin),
) -> Result<Event, ClientError>
where
   Resource: serde::de::DeserializeOwned,
   Event: TryFrom<WatchEvent<Resource>, Error = ClientError>,
{
   use futures_util::stream::StreamExt;

   loop {
      let new_bytes = stream
         .next()
         .await
         .ok_or(ClientError::WatchTimedOut)??;

      let pos = match new_bytes.iter().position(|byte| *byte == b'\n') {
         None => {
            event_builder.extend_from_slice(&new_bytes);
            continue;
         }
         Some(pos) => pos,
      };

      let mut new_bytes = Bytes::from(new_bytes.to_vec()).try_into_mut().unwrap();
      let rest_of_event = new_bytes.split_to(pos);

      event_builder.extend(rest_of_event);
      let string: String = event_builder.split().to_vec().try_into().unwrap();
      let event: WatchEvent<Resource> = serde_json::from_str::<WatchEvent<Resource>>(&string)?;

      assert!(event_builder.is_empty());
      event_builder.extend(new_bytes);

      return Ok(event.try_into()?);
   }
}


pub async fn get_stream(
   client: &Base,
   url: &str,
) -> Result<impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin, ClientError> {
   let stream = client
      .get(url)
      .send()
      .await?
      .error_for_status()?
      .bytes_stream();

   Ok(stream)
}
