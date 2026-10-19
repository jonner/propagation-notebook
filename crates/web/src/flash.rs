use serde::{Deserialize, Serialize};
use topcoat::{
    context::Cx,
    cookie::{CookieStore, Cookies, cookie_store, private_cookies},
    icon::{IconData, icon},
    view::{Attributes, View, component, view},
};

use crate::{assets::mdi, components::alert::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FlashMessage {
    Success(String),
    Failure(String),
}

impl FlashMessage {
    fn icon_data(&self) -> IconData {
        match self {
            FlashMessage::Success(_) => mdi::SUCCESS_BOLD,
            FlashMessage::Failure(_) => mdi::ERROR,
        }
    }

    fn variant(&self) -> AlertVariant {
        match self {
            FlashMessage::Success(_) => AlertVariant::Neutral,
            FlashMessage::Failure(_) => AlertVariant::Destructive,
        }
    }

    fn title(&self) -> &'static str {
        match self {
            FlashMessage::Success(_) => "Success",
            FlashMessage::Failure(_) => "Error",
        }
    }

    fn message(&self) -> &String {
        match self {
            FlashMessage::Success(msg) => msg,
            FlashMessage::Failure(msg) => msg,
        }
    }
}

#[component]
pub async fn flash_alert(
    flash: &FlashMessage,
    #[default] attrs: Attributes,
) -> topcoat::Result<impl View> {
    Ok(view! {
        alert(
            attrs: attrs,
            variant: flash.variant(),
            icon(data: flash.icon_data())
            alert_title((flash.title()))
            alert_description((flash.message()))
        )
    })
}

fn store(cx: &Cx) -> CookieStore<Option<FlashMessage>, impl Cookies> {
    cookie_store(private_cookies(cx).default_path("/"), "flash-message").parse_or_default()
}

pub fn push_message(cx: &Cx, message: FlashMessage) -> topcoat::Result<()> {
    store(cx).set(Some(message)).commit()?;
    Ok(())
}

pub fn pop_message(cx: &Cx) -> topcoat::Result<Option<FlashMessage>> {
    let store = store(cx);
    let msg = store.get();

    if msg.is_some() {
        store.remove();
    }

    Ok(msg)
}
