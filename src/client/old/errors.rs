use super::TargetInput;








#[derive(Debug)]
pub enum ClientError {
   Api(reqwest::Error),
   Serde(serde_json::Error),
   Json(&'static str),
   TargetValidationError(Vec<TargetError>),
   WatchTimedOut,
   SenderError(&'static str),
   ReceiverError(&'static str),
}


impl From<reqwest::Error> for ClientError {
   fn from(value: reqwest::Error) -> Self {
       ClientError::Api(value)
   }
}

impl From<serde_json::Error> for ClientError {
   fn from(value: serde_json::Error) -> Self {
      ClientError::Serde(value)
   }
}


impl std::fmt::Display for ClientError {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
         Self::Json(msg) => write!(f, "Json indexing error: {msg}"),
         Self::Api(e) => write!(f, "API error: {e}"),
         Self::Serde(e) => write!(f, "Json deserialisation error: {e}"),
         Self::WatchTimedOut => write!(f, "Watch request connection timed out"),
         Self::SenderError(msg) => write!(f, "Send error from: {msg}"),
         Self::ReceiverError(msg) => write!(f, "Receive error from: {msg}"),
         Self::TargetValidationError(errors) => {

            writeln!(f, "Targets Validation Error: some targets were invalid")?;

            for (i, error) in errors.iter().enumerate() {
               writeln!(f, "{i}: {error}")?;
            }

            Ok(())
         },
      }
       
   }

}


#[derive(Debug, Clone)]
pub enum ErrorKind {
   Namespace,
   Deployment,
   Containers(Vec<usize>)
}

#[derive(Debug, Clone)]
pub struct TargetError {
   pub kind: ErrorKind,
   pub target: TargetInput,
}


impl std::fmt::Display for TargetError {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      use std::fmt::Write;
      let deployment_name = self.target.deployment_name.as_ref();
      let namespace = self.target.namespace.as_ref();
      write!(f, "for the target \"/namespace/{namespace}/deployments/{deployment_name}\", ")?;

      match self.kind {
         ErrorKind::Deployment => write!(f, "the deployment was not found in the namespace"),
         ErrorKind::Namespace => write!(f, "the namespace was not found"),
         ErrorKind::Containers(ref indices) =>  {
            assert!(!self.target.containers.is_empty());

            let containers: Vec<&str> = indices.iter().map(|i| self.target.containers.get(*i).unwrap().as_ref()).collect();

            write!(f, "the following containers were not found: [")?;

            let mut s = String::new();

            for container in containers {
               write!(s, " \"{container}\",")?;

            };

            s.pop();


            write!(f, "{s} ]")
         },
      }
   }

}


