#![allow(clippy::module_name_repetitions)]

use maud::{html, Markup};
use moosicbox_core::sqlite::models::ApiTrack;

use crate::{public_img, state::State};

fn render_play_queue_item(track: &ApiTrack, is_history: bool) -> Markup {
    html! {
        div sx-dir="row" sx-opacity=[if is_history { Some(0.5) } else { None }] {
            (crate::albums::album_cover_img_from_track(track, 50))
            div {
                div {
                    (format!("{} - {}", track.title, track.album))
                }
                div {
                    (track.artist)
                }
            }
            @let icon_size = 20;
            button sx-width=(icon_size) sx-height=(icon_size) {
                img
                    sx-width=(icon_size)
                    sx-height=(icon_size)
                    src=(public_img!("cross-white.svg"));
            }
        }
    }
}

#[must_use]
pub fn play_queue(state: &State) -> Markup {
    static EMPTY_QUEUE: Vec<ApiTrack> = vec![];

    let position = state.player.playback.as_ref().map_or(0, |x| x.position);

    let queue: &[ApiTrack] = state
        .player
        .playback
        .as_ref()
        .map_or(&EMPTY_QUEUE, |x| &x.tracks);

    let history = queue
        .iter()
        .enumerate()
        .filter(|(i, _)| *i < position as usize)
        .map(|(_, x)| x);

    let mut future = queue
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= position as usize)
        .map(|(_, x)| x)
        .peekable();

    log::debug!("state: {state:?}");

    html! {
        div
            id="play-queue"
            sx-width="calc(min(500, 30%))"
            sx-height="calc(100% - 200)"
            sx-visibility="hidden"
            sx-background="#282a2b"
            sx-position="absolute"
            sx-bottom=(170)
            sx-right=(0)
        {
            h1 { ("Play queue") }
            @for track in history {
                (render_play_queue_item(track, true))
            }
            ({
                future.peek().map_or_else(|| html!(), |track| html! {
                    div sx-dir="row" {
                        ("Playing from: ")
                        a href=(crate::albums::album_page_url(&track.album_id.to_string(), false, Some(track.api_source), Some(track.track_source), track.sample_rate, track.bit_depth)) {
                            (track.album)
                        }
                    }
                })
            })
            @for track in future {
                (render_play_queue_item(track, false))
            }
        }
    }
}
