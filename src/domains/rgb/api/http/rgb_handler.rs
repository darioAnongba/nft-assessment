use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rgb_lib::wallet::{Assets, Metadata, ReceiveData, Transfer, Unspent};
use tracing::{debug, info, trace};

use crate::{
    adapters::rgb::RGBClient,
    application::{
        dtos::{
            InvoiceAssetRequest, IssueAssetRequest, PrepareIssuanceRequest, SendAssetsRequest,
            SendBTCRequest,
        },
        errors::ApplicationError,
    },
    domains::rgb::entities::{RGBAsset, RGBAssetType, RGBInvoiceType},
};

pub struct RGBHandler;

impl RGBHandler {
    pub fn routes() -> Router<Arc<dyn RGBClient>> {
        Router::new()
            .route("/assets", get(list_assets))
            .route("/assets/transfers", get(list_transfers))
            .route("/assets/issue", post(issue_asset))
            .route("/assets/invoice", post(invoice))
            .route("/assets/:id", get(get_asset))
            .route("/assets/:id/send", post(send_assets))
            .route("/assets/refresh", post(refresh))
            .route("/wallet/address", get(get_address))
            .route("/wallet/unspents", get(unspents))
            .route("/wallet/balance", get(get_balance))
            .route("/wallet/prepare-issuance", post(prepare_issuance))
            .route("/wallet/send", post(send))
    }
}

async fn get_address(State(rgb): State<Arc<dyn RGBClient>>) -> Result<String, ApplicationError> {
    trace!("Fetching address");

    let address = rgb.get_address().await?;

    debug!("Address fetched: {}", address);
    Ok(address)
}

async fn get_balance(State(rgb): State<Arc<dyn RGBClient>>) -> Result<String, ApplicationError> {
    trace!("Fetching balance");

    let balance = rgb.get_btc_balance().await?;

    debug!("Balance fetched: {}", balance);
    Ok(balance.to_string())
}

async fn unspents(
    State(rgb): State<Arc<dyn RGBClient>>,
) -> Result<Json<Vec<Unspent>>, ApplicationError> {
    trace!("Fetching unspents");

    let unspents = rgb.list_unspents().await?;

    debug!("Unspents fetched: {}", unspents.len());
    Ok(unspents.into())
}

async fn send(
    State(rgb): State<Arc<dyn RGBClient>>,
    Json(payload): Json<SendBTCRequest>,
) -> Result<String, ApplicationError> {
    info!(payload = ?payload, "Sending BTC");

    let tx_id = rgb
        .send_btc(payload.address.clone(), payload.amount, payload.fee_rate)
        .await?;

    info!(tx_id, recipient = payload.address, "BTC sent successfully");
    Ok(tx_id)
}

async fn prepare_issuance(
    State(rgb): State<Arc<dyn RGBClient>>,
    Json(payload): Json<PrepareIssuanceRequest>,
) -> Result<String, ApplicationError> {
    info!("Preparing utxos");

    let n_utxos = rgb.create_utxos(payload.fee_rate.unwrap_or(1.0)).await?;

    info!(n_utxos, "UTXOs created successfully");
    Ok(n_utxos.to_string())
}

async fn issue_asset(
    State(rgb): State<Arc<dyn RGBClient>>,
    Json(payload): Json<IssueAssetRequest>,
) -> Result<String, ApplicationError> {
    info!(asset_type = ?payload.asset_type, "Issuing asset");

    let amount = payload.amount.unwrap_or(1);

    let contract = RGBAsset {
        asset_type: payload.asset_type.clone(),
        ticker: payload.ticker.clone().unwrap_or("".to_string()),
        name: payload.name,
        details: payload.details,
        precision: payload.precision.unwrap_or(0),
        amounts: vec![amount],
        filename: payload.filename,
    };

    let asset_id = match payload.asset_type {
        RGBAssetType::NIA => rgb.issue_asset_nia(contract).await?,
        RGBAssetType::CFA => rgb.issue_asset_cfa(contract).await?,
        RGBAssetType::UDA => rgb.issue_asset_uda(contract).await?,
    };

    if let Some(recipient) = payload.recipient {
        let tx_id = rgb
            .send(
                asset_id.clone(),
                recipient,
                true,
                payload.fee_rate.unwrap_or(1.0),
                amount,
            )
            .await?;

        info!(asset_id, tx_id, "Asset issued and sent successfully");
    } else {
        info!(asset_id, "Asset issued successfully");
    }

    Ok(asset_id)
}

async fn list_assets(
    State(rgb): State<Arc<dyn RGBClient>>,
) -> Result<Json<Assets>, ApplicationError> {
    trace!("Fetching assets");

    let assets = rgb.list_assets().await?;

    Ok(assets.into())
}

async fn list_transfers(
    State(rgb): State<Arc<dyn RGBClient>>,
) -> Result<Json<Vec<Transfer>>, ApplicationError> {
    trace!("Fetching asset transfers");

    let assets = rgb.list_transfers(None).await?;

    Ok(assets.into())
}

async fn get_asset(
    Path(id): Path<String>,
    State(rgb): State<Arc<dyn RGBClient>>,
) -> Result<Json<Metadata>, ApplicationError> {
    trace!(id, "Fetching asset");

    let asset = rgb.get_asset(id).await?;

    Ok(asset.into())
}

async fn send_assets(
    Path(id): Path<String>,
    State(rgb): State<Arc<dyn RGBClient>>,
    Json(payload): Json<SendAssetsRequest>,
) -> Result<String, ApplicationError> {
    info!(
        asset_id = id,
        recipient = payload.recipient,
        "Sending assets"
    );

    let tx_id = rgb
        .send(
            id,
            payload.recipient,
            true,
            payload.fee_rate.unwrap_or(1.0),
            payload.amount.unwrap_or(1),
        )
        .await?;

    info!(tx_id, "Assets sent successfully");
    Ok(tx_id)
}

async fn invoice(
    State(rgb): State<Arc<dyn RGBClient>>,
    Json(payload): Json<InvoiceAssetRequest>,
) -> Result<Json<ReceiveData>, ApplicationError> {
    info!(invoice_type = ?payload.invoice_type, "Generating invoice");

    let invoice = match payload.invoice_type {
        RGBInvoiceType::BLIND => {
            rgb.blind_receive(payload.asset_id, payload.amount, payload.duration_seconds)
                .await?
        }
        RGBInvoiceType::WITNESS => {
            rgb.witness_receive(payload.asset_id, payload.amount, payload.duration_seconds)
                .await?
        }
    };

    info!(invoice = invoice.invoice, "Invoice generated successfully");
    Ok(invoice.into())
}

async fn refresh(State(rgb): State<Arc<dyn RGBClient>>) -> impl IntoResponse {
    info!("Refreshing asset transfers");

    match rgb.refresh(None).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => e.into_response(),
    }
}
