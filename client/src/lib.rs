use terminusdb_grpc_labelstore_proto as proto;

use bytes::Bytes;
use proto::{LayerId, SetLabelRequest};
use std::sync::Arc;
use terminus_store::storage::{Label, LabelStore};
use terminusdb_grpc_labelstore_proto::{
    label_service_client::LabelServiceClient, CreateLabelRequest, DeleteLabelRequest,
    GetLabelRequest, GetLabelsRequest,
};
use tokio::sync::Mutex;
use tonic::{
    transport::{Channel, Endpoint},
    Code,
};

use async_trait::*;
use std::io;

#[derive(Clone)]
pub struct GrpcLabelStore {
    state: Arc<GrpcLabelStoreState>,
}

impl GrpcLabelStore {
    pub async fn new(
        address: impl Into<Bytes>,
        init_channels: usize,
    ) -> Result<Self, tonic::transport::Error> {
        let state = Arc::new(GrpcLabelStoreState::new(address, init_channels).await?);
        Ok(Self { state })
    }
}

pub struct GrpcLabelStoreState {
    address: Bytes,
    clients: Mutex<Vec<LabelServiceClient<Channel>>>,
}

impl GrpcLabelStoreState {
    pub async fn new(
        address: impl Into<Bytes>,
        init_channels: usize,
    ) -> Result<Self, tonic::transport::Error> {
        let mut clients = Vec::new();
        let shared = address.into();
        for _ in 0..init_channels {
            let client = new_client(shared.clone()).await?;
            clients.push(client);
        }

        Ok(Self {
            address: shared,
            clients: Mutex::new(clients),
        })
    }
}

async fn new_client(
    address: Bytes,
) -> Result<LabelServiceClient<Channel>, tonic::transport::Error> {
    LabelServiceClient::connect(Endpoint::from_shared(address)?).await
}

impl GrpcLabelStoreState {
    async fn get_available_client(&self) -> Result<LabelServiceClient<Channel>, io::Error> {
        let mut clients = self.clients.lock().await;
        if let Some(client) = clients.pop() {
            Ok(client)
        } else {
            match new_client(self.address.clone()).await {
                Ok(client) => Ok(client),
                Err(e) => Err(io::Error::new(io::ErrorKind::NotFound, e.to_string())),
            }
        }
    }

    async fn store_client(&self, client: LabelServiceClient<Channel>) {
        let mut clients = self.clients.lock().await;
        clients.push(client);
    }
}

#[async_trait]
impl LabelStore for GrpcLabelStore {
    async fn labels(&self) -> io::Result<Vec<Label>> {
        let mut client = self.state.get_available_client().await?;
        let response = client.get_labels(GetLabelsRequest::default()).await;

        let result = match response {
            Ok(response) => {
                let r = response.into_inner();

                let labels = r
                    .label
                    .into_iter()
                    .map(|l| Label {
                        name: l.name,
                        layer: l.layer.map(|l| l.id()),
                        version: l.version,
                    })
                    .collect();

                Ok(labels)
            }
            Err(status) => {
                eprintln!("encountered error: {status}");
                Err(io::Error::new(io::ErrorKind::Other, status.message()))
            }
        };

        self.state.store_client(client).await;

        result
    }

    async fn create_label(&self, label: &str) -> io::Result<Label> {
        let mut client = self.state.get_available_client().await?;
        let response = client
            .create_label(CreateLabelRequest {
                domain: "".to_string(),
                name: label.to_string(),
            })
            .await;

        let result = match response {
            Ok(response) => {
                let r = response.into_inner();

                if r.has_been_created {
                    Ok(Label::new_empty(label))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "database already exists",
                    ))
                }
            }
            Err(status) => {
                eprintln!("encountered error: {status}");
                Err(io::Error::new(io::ErrorKind::Other, status.message()))
            }
        };

        self.state.store_client(client).await;

        result
    }

    async fn get_label(&self, label: &str) -> io::Result<Option<Label>> {
        let mut client = self.state.get_available_client().await?;
        let response = client
            .get_label(GetLabelRequest {
                domain: "".to_string(),
                name: label.to_string(),
            })
            .await;

        let result = match response {
            Ok(response) => {
                let r = response.into_inner();

                Ok(Some(Label {
                    name: label.to_string(),
                    layer: r.id(),
                    version: r.version,
                }))
            }
            Err(status) => {
                if status.code() == Code::NotFound {
                    Ok(None)
                } else {
                    eprintln!("encountered error: {status}");
                    Err(io::Error::new(io::ErrorKind::Other, status.message()))
                }
            }
        };

        self.state.store_client(client).await;

        result
    }

    async fn set_label_option(
        &self,
        label: &Label,
        layer: Option<[u32; 5]>,
    ) -> io::Result<Option<Label>> {
        let mut client = self.state.get_available_client().await?;
        let request_label = proto::Label {
            name: label.name.clone(),
            layer: label.layer.map(LayerId::new),
            version: label.version,
        };
        let response = client
            .set_label(SetLabelRequest {
                domain: "".to_string(),
                label: Some(request_label),
                new_layer: layer.map(LayerId::new),
            })
            .await;

        let result = match response {
            Ok(response) => {
                let r = response.into_inner();

                let new_layer = if r.has_been_set { layer } else { label.layer };
                Ok(Some(Label {
                    name: label.name.clone(),
                    layer: new_layer,
                    version: r.version,
                }))
            }
            Err(status) => {
                if status.code() == Code::NotFound {
                    Ok(None)
                } else {
                    eprintln!("encountered error: {status}");
                    Err(io::Error::new(io::ErrorKind::Other, status.message()))
                }
            }
        };

        self.state.store_client(client).await;

        result
    }

    async fn delete_label(&self, name: &str) -> io::Result<bool> {
        let mut client = self.state.get_available_client().await?;
        let response = client
            .delete_label(DeleteLabelRequest {
                domain: "".to_string(),
                name: name.to_string(),
            })
            .await;

        let result = match response {
            Ok(response) => {
                let r = response.into_inner();

                Ok(r.has_been_deleted)
            }
            Err(status) => {
                eprintln!("encountered error: {status}");
                Err(io::Error::new(io::ErrorKind::Other, status.message()))
            }
        };

        self.state.store_client(client).await;

        result
    }
}
