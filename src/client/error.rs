use reqwest::Error;
use super::KubeErrorStatus;

#[derive(Debug)]
pub enum JsonQuery
{
   NoUid,
   NoMetaData,
   NoResourceVersion,
   NoStatus,
   NoCondition,
   NoReadyCondition,
   NoName,
}

#[derive(Debug)]
pub enum Resource
{
   DaemonSet,
}

#[derive(Debug)]
pub enum ReceiverError {}

#[derive(Debug)]
pub enum APIError
{
   Http(Error),
   Response(KubeErrorStatus),
   JsonParse(serde_json::Error),
   JsonQuery(JsonQuery),
   WatcherEventReceiver
   {
      resource: Resource,
      issue: ReceiverError,
   },
   ChannelReceiverDropped,
   ChannelSenderDropped,
   WatcherTermination,
}

impl From<Error> for APIError
{
   fn from(value: Error) -> Self
   {
      Self::Http(value)
   }
}

impl From<serde_json::Error> for APIError
{
   fn from(value: serde_json::Error) -> Self
   {
      Self::JsonParse(value)
   }
}

impl From<KubeErrorStatus> for APIError
{
   fn from(value: KubeErrorStatus) -> Self
   {
      Self::Response(value)
   }
}

pub async fn response_into_error(response: reqwest::Response) -> Result<reqwest::Response, APIError>
{
   if response.status() == reqwest::StatusCode::OK {
      return Ok(response);
   };

   let error = match response.json::<KubeErrorStatus>().await {
         Err(e) => e.into(),
         Ok(status) => status.into(),
   };

   Err(error)
}


pub mod errors {
   use super::{APIError, JsonQuery};
   pub const ERR_RESOURCE: APIError = APIError::JsonQuery(JsonQuery::NoResourceVersion);
   pub const UID: APIError = APIError::JsonQuery(JsonQuery::NoUid);
   pub const STATUS: APIError = APIError::JsonQuery(JsonQuery::NoStatus);
   pub const CONDITION: APIError = APIError::JsonQuery(JsonQuery::NoCondition);
   pub const READY: APIError = APIError::JsonQuery(JsonQuery::NoReadyCondition);
   pub const NAME: APIError = APIError::JsonQuery(JsonQuery::NoName);
}

