use std::sync::Arc;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_lsp_server::ls_types::notification::Notification;

use crate::{
    bridge::router::{DefaultingRouter, Router},
    layer::{client::LanguageClient, server::LsServer},
    transport::error::ExitedError,
};

pub struct JdtlsClient<S> {
    #[allow(dead_code)]
    client: LsServer<S>,
    status: Arc<RwLock<LanguageStatus>>,
}

impl<S: Clone> Clone for JdtlsClient<S> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            status: self.status.clone(),
        }
    }
}

impl<S> JdtlsClient<S> {
    pub fn new(client: LsServer<S>) -> Self {
        Self {
            client,
            status: Arc::new(RwLock::new(LanguageStatus::Starting)),
        }
    }

    pub fn into_router(self) -> DefaultingRouter<Self, LsServer<S>, ExitedError>
    where
        S: Clone + Send + Sync + 'static,
    {
        let client = self.client.clone();
        let mut result = Router::new(self);
        result.method(
            LanguageStatus::METHOD,
            Self::status,
            tower::layer::util::Identity::new(),
        );
        DefaultingRouter::new(result, client)
    }

    pub async fn get_status(&self) -> LanguageStatus {
        self.status.read().await.clone()
    }

    pub async fn status(&self, params: LanguageStatusParams) {
        Self::status_const(self.status.clone(), params).await
    }

    async fn status_const(status: Arc<RwLock<LanguageStatus>>, params: LanguageStatusParams) {
        if let Ok(status_type) = params._type.try_into() {
            let mut status = status.clone().write_owned().await;
            if *status < status_type {
                *status = status_type;
            }
        }
        match params._type {
            LanguageStatusType::Error => error!("Jdtls Error: {}", &params.message),
            s => info!(
                "Jdtls Status {}: {}",
                serde_json::to_string(&s).unwrap(),
                &params.message
            ),
        }
    }
}

impl<S> LanguageClient for JdtlsClient<S> where S: Send + Sync + 'static {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LanguageStatus {
    Starting,
    Started,
    Ready,
}

impl TryFrom<LanguageStatusType> for LanguageStatus {
    type Error = ();

    fn try_from(value: LanguageStatusType) -> Result<Self, Self::Error> {
        match value {
            LanguageStatusType::Starting => Ok(Self::Starting),
            LanguageStatusType::Started => Ok(Self::Started),
            LanguageStatusType::ServiceReady => Ok(Self::Ready),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LanguageStatusParams {
    pub message: String,

    #[serde(rename = "type")]
    pub _type: LanguageStatusType,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum LanguageStatusType {
    Starting,
    Started,
    ServiceReady,
    Error,
    ProjectStatus,
    Message,
}

impl Notification for LanguageStatus {
    type Params = LanguageStatusParams;

    const METHOD: &'static str = "language/status";
}
