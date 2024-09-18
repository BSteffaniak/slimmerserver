use moosicbox_session::events::BoxErrorSend;

use crate::{CONFIG_DB, DB, WS_SERVER_HANDLE};

pub async fn init() {
    moosicbox_session::events::on_players_updated_event({
        move || async move {
            log::debug!("on_players_updated_event: Players updated");
            let connection_id = "self";
            let context = moosicbox_ws::WebsocketContext {
                connection_id: connection_id.to_string(),
                ..Default::default()
            };
            let handle = WS_SERVER_HANDLE
                .read()
                .await
                .clone()
                .ok_or(moosicbox_ws::WebsocketSendError::Unknown(
                    "No ws server handle".into(),
                ))
                .map_err(|e| Box::new(e) as BoxErrorSend)?;
            let db = { DB.read().unwrap().clone().unwrap() };
            moosicbox_ws::get_sessions(&db, &handle, &context, true)
                .await
                .map_err(|e| Box::new(e) as BoxErrorSend)?;
            let config_db = { CONFIG_DB.read().unwrap().clone().unwrap() };
            moosicbox_ws::broadcast_connections(&config_db, &handle)
                .await
                .map_err(|e| Box::new(e) as BoxErrorSend)?;
            Ok(())
        }
    })
    .await;
}
