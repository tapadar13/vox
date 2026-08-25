use std::{thread, time::Duration};

use arboard::{Clipboard, ImageData};
use async_trait::async_trait;

use crate::{
    error::{VoxError, VoxResult},
    ports::{DeliveryOutcome, TextDelivery},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct MacTextDelivery;

enum ClipboardBackup {
    Text(String),
    Image(ImageData<'static>),
    Empty,
}

#[async_trait]
impl TextDelivery for MacTextDelivery {
    async fn deliver(&self, text: &str, auto_paste: bool) -> VoxResult<DeliveryOutcome> {
        let text = text.to_owned();
        tokio::task::spawn_blocking(move || deliver_blocking(&text, auto_paste))
            .await
            .map_err(|error| VoxError::Delivery(error.to_string()))?
    }
}

fn deliver_blocking(text: &str, auto_paste: bool) -> VoxResult<DeliveryOutcome> {
    let mut clipboard = Clipboard::new().map_err(|error| VoxError::Delivery(error.to_string()))?;
    let backup = backup_clipboard(&mut clipboard);
    clipboard
        .set_text(text)
        .map_err(|error| VoxError::Delivery(error.to_string()))?;

    if !auto_paste || !accessibility_granted() {
        return Ok(DeliveryOutcome::Clipboard);
    }
    if synthesize_paste().is_err() {
        return Ok(DeliveryOutcome::Clipboard);
    }

    thread::sleep(Duration::from_millis(180));
    if let Err(error) = restore_clipboard(&mut clipboard, backup) {
        tracing::warn!(%error, "could not restore the previous clipboard after paste");
    }
    Ok(DeliveryOutcome::Pasted)
}

fn backup_clipboard(clipboard: &mut Clipboard) -> ClipboardBackup {
    if let Ok(text) = clipboard.get_text() {
        ClipboardBackup::Text(text)
    } else if let Ok(image) = clipboard.get_image() {
        ClipboardBackup::Image(image)
    } else {
        ClipboardBackup::Empty
    }
}

fn restore_clipboard(clipboard: &mut Clipboard, backup: ClipboardBackup) -> VoxResult<()> {
    match backup {
        ClipboardBackup::Text(text) => clipboard.set_text(text),
        ClipboardBackup::Image(image) => clipboard.set_image(image),
        ClipboardBackup::Empty => clipboard.clear(),
    }
    .map_err(|error| VoxError::Delivery(error.to_string()))
}

#[cfg(target_os = "macos")]
fn accessibility_granted() -> bool {
    unsafe extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
    }

    // SAFETY: CoreGraphics exposes this parameter-free permission query on macOS 10.15+.
    unsafe { CGPreflightPostEventAccess() }
}

#[cfg(not(target_os = "macos"))]
fn accessibility_granted() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn synthesize_paste() -> VoxResult<()> {
    use core_graphics::{
        event::{CGEvent, CGEventFlags, CGEventTapLocation, KeyCode},
        event_source::{CGEventSource, CGEventSourceStateID},
    };

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| VoxError::Delivery("could not create a keyboard event source".to_owned()))?;
    let key_down = CGEvent::new_keyboard_event(source.clone(), KeyCode::ANSI_V, true)
        .map_err(|_| VoxError::Delivery("could not create paste key-down event".to_owned()))?;
    let key_up = CGEvent::new_keyboard_event(source, KeyCode::ANSI_V, false)
        .map_err(|_| VoxError::Delivery("could not create paste key-up event".to_owned()))?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(CGEventTapLocation::HID);
    key_up.post(CGEventTapLocation::HID);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn synthesize_paste() -> VoxResult<()> {
    Err(VoxError::Delivery(
        "automatic paste is only implemented on macOS".to_owned(),
    ))
}
