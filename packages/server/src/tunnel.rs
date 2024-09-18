use std::env;

use moosicbox_auth::get_client_id_and_access_token;
use moosicbox_database::config::ConfigDatabase;
use moosicbox_music_api::MusicApiState;
use moosicbox_tunnel::TunnelRequest;
use moosicbox_tunnel_sender::{
    sender::{TunnelSender, TunnelSenderHandle},
    TunnelMessage,
};
use thiserror::Error;
use tokio::task::JoinHandle;
use url::Url;

use crate::{CANCELLATION_TOKEN, WS_SERVER_HANDLE};

#[derive(Debug, Error)]
pub enum SetupTunnelError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[allow(clippy::too_many_lines, clippy::module_name_repetitions)]
pub async fn setup_tunnel(
    config_db: ConfigDatabase,
    music_apis: MusicApiState,
    service_port: u16,
) -> Result<
    (
        Option<String>,
        Option<JoinHandle<()>>,
        Option<TunnelSenderHandle>,
    ),
    SetupTunnelError,
> {
    if let Ok(url) = env::var("WS_HOST") {
        if url.is_empty() {
            Ok((None, None, None))
        } else {
            log::debug!("Using WS_HOST: {url}");
            let ws_url = url.clone();
            let url = Url::parse(&url).expect("Invalid WS_HOST");
            let hostname = url
                .host_str()
                .map(std::string::ToString::to_string)
                .expect("Invalid WS_HOST");
            let host = format!(
                "{}://{hostname}{}",
                if url.scheme() == "wss" {
                    "https"
                } else {
                    "http"
                },
                url.port()
                    .map_or_else(String::new, |port| format!(":{port}"))
            );
            // FIXME: Handle retry
            let (client_id, access_token) = {
                get_client_id_and_access_token(&config_db, &host)
                    .await
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Could not get access token: {e:?}"),
                        )
                    })?
            };
            let (mut tunnel, handle) = TunnelSender::new(
                host.clone(),
                ws_url,
                client_id,
                access_token,
                config_db.clone(),
            );

            tunnel = tunnel.with_cancellation_token(CANCELLATION_TOKEN.clone());

            let music_apis_send = music_apis.clone();
            Ok((
                Some(host),
                Some(moosicbox_task::spawn("server: tunnel", async move {
                    let mut rx = tunnel.start();

                    while let Some(m) = rx.recv().await {
                        match m {
                            TunnelMessage::Text(m) => {
                                log::debug!("Received text TunnelMessage {}", &m);
                                let tunnel = tunnel.clone();
                                let music_apis_send = music_apis_send.clone();
                                moosicbox_task::spawn("server: tunnel message", async move {
                                    match serde_json::from_str(&m).unwrap() {
                                        TunnelRequest::Http(request) => {
                                            if let Err(err) = tunnel
                                                .tunnel_request(
                                                    music_apis_send.apis,
                                                    service_port,
                                                    request.request_id,
                                                    request.method,
                                                    request.path,
                                                    request.query,
                                                    request.payload,
                                                    request.headers,
                                                    request.profile,
                                                    request.encoding,
                                                )
                                                .await
                                            {
                                                log::error!("Tunnel request failed: {err:?}");
                                            }
                                        }
                                        TunnelRequest::Ws(request) => {
                                            let sender = WS_SERVER_HANDLE
                                                .read()
                                                .await
                                                .as_ref()
                                                .ok_or("Failed to get ws server handle")?
                                                .clone();
                                            if let Err(err) = tunnel
                                                .ws_request(
                                                    request.conn_id,
                                                    request.request_id,
                                                    request.body.clone(),
                                                    request.profile,
                                                    sender,
                                                )
                                                .await
                                            {
                                                log::error!(
                                                                "Failed to propagate ws request {} from conn_id {}: {err:?}",
                                                                request.request_id,
                                                                request.conn_id
                                                            );
                                            }
                                        }
                                        TunnelRequest::Abort(request) => {
                                            log::debug!("Aborting request {}", request.request_id);
                                            tunnel.abort_request(request.request_id);
                                        }
                                    }
                                    Ok::<_, String>(())
                                });
                            }
                            TunnelMessage::Binary(_) => {
                                unimplemented!("Binary TunnelMessage is not implemented")
                            }
                            TunnelMessage::Ping(_) | TunnelMessage::Pong(_) => {}
                            TunnelMessage::Close => {
                                log::info!("Tunnel connection was closed");
                            }
                            TunnelMessage::Frame(_) => {
                                unimplemented!("Frame TunnelMessage is not implemented")
                            }
                        }
                    }
                    log::debug!("Exiting tunnel message loop");
                })),
                Some(handle),
            ))
        }
    } else {
        Ok((None, None, None))
    }
}
