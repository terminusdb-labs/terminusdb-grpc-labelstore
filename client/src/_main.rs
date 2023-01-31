use terminusdb_labels_proto as proto;
use tonic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut client = proto::labels_client::LabelsClient::connect("http://[::1]:8080").await?;
    for _ in 0..100000 {
        let request = tonic::Request::new(proto::GetLabelRequest { name: "terminusdb%3a%2f%2f%2fsystem%2fdata".to_string(), domain: "".to_string() });

        let response = client.get_label(request).await?;
        assert!(response.into_inner().id().is_some());
    }

    Ok(())
}
