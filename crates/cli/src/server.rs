use crate::convert::{self, ConvertError};
use axum::{
    Json, Router,
    extract::{Multipart, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use wf_core::analysis::{AnalysisError, AnalysisOptions, AnalysisReport, PropertyResult, analyze};
use wf_core::petri_net::{
    exporter::{ExportOptions, export_to_dot_with_options, export_to_pnml_with_options},
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
    /// `lr` lays the DOT graph out from left to right.
    rankdir: Option<String>,
    /// Set to `false` to omit the initial marking.
    initial_marking: Option<bool>,
}

impl ConvertQuery {
    fn export_options(&self) -> ExportOptions {
        ExportOptions {
            initial_marking: self.initial_marking.unwrap_or(true),
            rankdir_lr: self
                .rankdir
                .as_deref()
                .is_some_and(|rankdir| rankdir.eq_ignore_ascii_case("lr")),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AnalyzeQuery {
    /// Path to TINA's `sift` binary.
    sift: Option<String>,
    max_states: Option<usize>,
    timeout: Option<u64>,
}

#[derive(Serialize)]
struct BothResponse {
    pnml: String,
    dot: String,
}

#[derive(Serialize)]
struct PropertyResponse {
    property: String,
    status: &'static str,
    violation: Option<String>,
    states: Option<usize>,
    log: String,
}

impl From<&PropertyResult> for PropertyResponse {
    fn from(result: &PropertyResult) -> Self {
        Self {
            property: result.property.label().to_string(),
            status: result.outcome.label(),
            violation: result.violation.clone(),
            states: result.states,
            log: result.output.clone(),
        }
    }
}

#[derive(Serialize)]
struct AnalyzeResponse {
    places: usize,
    transitions: usize,
    all_hold: bool,
    safety: PropertyResponse,
    deadlock: PropertyResponse,
}

impl From<&AnalysisReport> for AnalyzeResponse {
    fn from(report: &AnalysisReport) -> Self {
        Self {
            places: report.places,
            transitions: report.transitions,
            all_hold: report.all_hold(),
            safety: PropertyResponse::from(&report.safety),
            deadlock: PropertyResponse::from(&report.deadlock),
        }
    }
}

#[derive(Debug)]
enum ServerError {
    MissingFile,
    Multipart(String),
    Convert(ConvertError),
    Analysis(AnalysisError),
}

impl From<ConvertError> for ServerError {
    fn from(e: ConvertError) -> Self {
        ServerError::Convert(e)
    }
}

impl From<AnalysisError> for ServerError {
    fn from(e: AnalysisError) -> Self {
        ServerError::Analysis(e)
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
            ServerError::Analysis(e) => (StatusCode::FAILED_DEPENDENCY, e.to_string()),
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

async fn analyze_blocking(
    net: PetriNet,
    options: AnalysisOptions,
) -> Result<AnalysisReport, AnalysisError> {
    tokio::task::spawn_blocking(move || analyze(&net, &options))
        .await
        .expect("analysis task panicked")
}

fn export_response(
    net: &PetriNet,
    format: Format,
    options: ExportOptions,
) -> Result<Response, ConvertError> {
    match format {
        Format::Pnml => {
            let body = export_to_pnml_with_options(net, options)?;
            Ok(([(header::CONTENT_TYPE, "application/xml")], body).into_response())
        }
        Format::Dot => {
            let body = export_to_dot_with_options(net, options)?;
            Ok(([(header::CONTENT_TYPE, "text/vnd.graphviz")], body).into_response())
        }
        Format::Both => {
            let pnml = export_to_pnml_with_options(net, options)?;
            let dot = export_to_dot_with_options(net, options)?;
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
    export_response(
        &net,
        query.format.unwrap_or(Format::Both),
        query.export_options(),
    )
    .map_err(ServerError::from)
}

async fn analyze_endpoint(
    Query(query): Query<AnalyzeQuery>,
    multipart: Multipart,
) -> Result<Json<AnalyzeResponse>, ServerError> {
    let xml = extract_bpmn_file(multipart).await?;
    let net = encode_blocking(xml).await?;

    let options = AnalysisOptions {
        sift_path: query.sift.unwrap_or_else(|| "sift".to_string()),
        max_states: query.max_states,
        time_limit: query.timeout,
    };

    let report = analyze_blocking(net, options).await?;
    Ok(Json(AnalyzeResponse::from(&report)))
}

pub fn run(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;

    Ok(runtime.block_on(async move {
        let app = Router::new()
            .route("/convert", post(convert_endpoint))
            .route("/analyze", post(analyze_endpoint));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        println!("Server listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await
    })?)
}
