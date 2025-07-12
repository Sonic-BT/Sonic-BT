use core::str::FromStr;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use heapless::String;
use log::{error, info, warn};

type Message = Option<String<128>>;

pub enum Status {
    Ok(Message),
    Warn(Message),
    Err(Message),
}

/// # Status Signal
/// This signal is avalible between all threads to
/// detect system status at the same time.
pub static STATUS: Signal<CriticalSectionRawMutex, Status> = Signal::new();

/// # Check Message Contents
/// if message contents arn't a empty string
/// e.g. "" then include in log message.
fn check_message(message: Message) -> String<132> {
    let mut format_string: String<132> =
        String::from_str(": ").expect("Cannot create [`heapless::String`] from [`str`]");

    match message {
        Some(message) => {
            format_string
                .push_str(message.as_str())
                .expect("Cannot push_str message tring to end of the [`heapless::String`]");
            return format_string;
        }
        None => String::new(),
    }
}

#[embassy_executor::task]
pub async fn run() -> () {
    loop {
        let led_status: Status = STATUS.wait().await;

        match led_status {
            Status::Ok(message) => info!("[ Okay  ]{}", check_message(message)),
            Status::Warn(message) => warn!("[ Warn  ]{}", check_message(message)),
            Status::Err(message) => error!("[ Panic ]{}", check_message(message)),
        }
    }
}
