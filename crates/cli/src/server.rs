use crate::convert::{self, ConvertError};
use axum::{
    Json, Router,
    extract::{Multipart, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use wf_core::petri_net::{
    exporter::{export_to_dot, export_to_pnml},
    pn::PetriNet,
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Format {
    Pnml,
    Dot,
    Both,
}

#[derive(Debug, Deserialize)]
struct ConvertQuery {
    format: Option<Format>,
}

#[derive(Serialize)]
struct BothResponse {
    pnml: String,
    dot: String,
}

#[derive(Debug)]
enum ServerError {
    MissingFile,
    Multipart(String),
    Convert(ConvertError),
}

impl From<ConvertError> for ServerError {
    fn from(e: ConvertError) -> Self {
        ServerError::Convert(e)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::MissingFile => {
                (StatusCode::BAD_REQUEST, "missing 'file' field".to_string())
            }
            ServerError::Multipart(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::Convert(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
        };
        (status, message).into_response()
    }
}

async fn extract_bpmn_file(mut multipart: Multipart) -> Result<String, ServerError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServerError::Multipart(e.to_string()))?
    {
        if field.name() == Some("file") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ServerError::Multipart(e.to_string()))?;
            return Ok(String::from_utf8_lossy(&bytes).into_owned());
        }
    }
    Err(ServerError::MissingFile)
}

async fn encode_blocking(xml: String) -> Result<PetriNet, ConvertError> {
    tokio::task::spawn_blocking(move || convert::xml_to_net(&xml))
        .await
        .expect("encode task panicked")
}

fn export_response(net: &PetriNet, format: Format) -> Result<Response, ConvertError> {
    match format {
        Format::Pnml => {
            let body = export_to_pnml(net)?;
            Ok(([(header::CONTENT_TYPE, "application/xml")], body).into_response())
        }
        Format::Dot => {
            let body = export_to_dot(net)?;
            Ok(([(header::CONTENT_TYPE, "text/vnd.graphviz")], body).into_response())
        }
        Format::Both => {
            let pnml = export_to_pnml(net)?;
            let dot = export_to_dot(net)?;
            Ok(Json(BothResponse { pnml, dot }).into_response())
        }
    }
}

async fn convert_endpoint(
    Query(query): Query<ConvertQuery>,
    multipart: Multipart,
) -> Result<Response, ServerError> {
    let xml = extract_bpmn_file(multipart).await?;
    let net = encode_blocking(xml).await?;
    export_response(&net, query.format.unwrap_or(Format::Both)).map_err(ServerError::from)
}

pub fn run(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;

    Ok(runtime.block_on(async move {
        let app = Router::new().route("/convert", post(convert_endpoint));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        println!("Server listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await
    })?)
}
