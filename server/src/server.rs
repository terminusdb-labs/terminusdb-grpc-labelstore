use terminus_store::storage::{Label, LabelStore};
use terminusdb_grpc_labelstore_proto as proto;
use tonic::{transport::Server, Request, Response, Status};

pub struct WrappedStoreLabelService<S> {
    store: S,
}

#[tonic::async_trait]
impl<S: LabelStore + 'static> proto::label_service_server::LabelService
    for WrappedStoreLabelService<S>
{
    async fn get_label(
        &self,
        request: Request<proto::GetLabelRequest>,
    ) -> Result<Response<proto::GetLabelResponse>, Status> {
        let r = request.into_inner();

        if let Some(label) = self.store.get_label(&r.name).await? {
            Ok(Response::new(proto::GetLabelResponse::new(
                label.layer,
                label.version,
            )))
        } else {
            Err(Status::not_found(format!("label not found")))
        }
    }

    async fn get_labels(
        &self,
        _request: Request<proto::GetLabelsRequest>,
    ) -> Result<Response<proto::GetLabelsResponse>, Status> {
        let labels: Vec<_> = self
            .store
            .labels()
            .await?
            .into_iter()
            .map(|l| proto::Label::new(l.name, l.layer, l.version))
            .collect();

        Ok(Response::new(proto::GetLabelsResponse { label: labels }))
    }

    async fn create_label(
        &self,
        request: Request<proto::CreateLabelRequest>,
    ) -> Result<Response<proto::CreateLabelResponse>, Status> {
        let r = request.into_inner();
        let has_been_created;
        if let Err(e) = self.store.create_label(&r.name).await {
            // invalidinput is given for labels that already exist
            if e.kind() == std::io::ErrorKind::InvalidInput {
                has_been_created = false;
            } else {
                return Err(e.into());
            }
        } else {
            has_been_created = true;
        }
        Ok(Response::new(proto::CreateLabelResponse {
            has_been_created,
        }))
    }

    async fn set_label(
        &self,
        request: Request<proto::SetLabelRequest>,
    ) -> Result<Response<proto::SetLabelResponse>, Status> {
        let r = request.into_inner();

        if let Some(label) = r.label {
            let inner_label = Label {
                name: label.name,
                layer: label.layer.map(|l| l.id()),
                version: label.version,
            };

            if let Some(new_label) = self
                .store
                .set_label_option(&inner_label, r.new_layer.map(|l| l.id()))
                .await?
            {
                Ok(Response::new(proto::SetLabelResponse {
                    has_been_set: true,
                    version: new_label.version,
                }))
            } else {
                Ok(Response::new(proto::SetLabelResponse {
                    has_been_set: false,
                    version: label.version,
                }))
            }
        } else {
            Err(Status::invalid_argument("label argument not specified"))
        }
    }

    async fn delete_label(
        &self,
        request: Request<proto::DeleteLabelRequest>,
    ) -> Result<Response<proto::DeleteLabelResponse>, Status> {
        let r = request.into_inner();

        if r.name.is_empty() {
            return Err(Status::invalid_argument("no label name given"));
        }

        let has_been_deleted = self.store.delete_label(&r.name).await?;

        Ok(Response::new(proto::DeleteLabelResponse {
            has_been_deleted,
        }))
    }
}

pub fn create_service<S: LabelStore + 'static>(
    store: S,
) -> proto::label_service_server::LabelServiceServer<WrappedStoreLabelService<S>> {
    let service = WrappedStoreLabelService { store };
    proto::label_service_server::LabelServiceServer::new(service)
}

pub async fn spawn_server<S: LabelStore + 'static>(
    store: S,
    port: u16,
) -> Result<(), tonic::transport::Error> {
    Server::builder()
        .add_service(create_service(store))
        .serve(format!("[::]:{port}").parse().unwrap())
        .await
}
