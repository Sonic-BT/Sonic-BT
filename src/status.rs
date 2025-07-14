use core::str::FromStr;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use heapless::String;
use log::{error, info, warn};

type Message = Option<String<128>>;

#[derive(Clone)]
pub enum Status {
    Ok(Message),
    Warn(Message),
    Err(Message),
}

/// # Status Signal
/// This signal is avalible between all threads to
/// detect system status at the same time.
pub static STATUS: PubSubChannel<CriticalSectionRawMutex, Status, 2, 2, 4> = PubSubChannel::new();

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
            format_string
        }
        None => String::new(),
    }
}

#[embassy_executor::task]
pub async fn run() -> () {
    let mut sub = STATUS.subscriber().unwrap();

    loop {
        let status: Status = sub.next_message_pure().await;

        match status {
            Status::Ok(message) => info!("[ Okay  ]{}", check_message(message)),
            Status::Warn(message) => warn!("[ Warn  ]{}", check_message(message)),
            Status::Err(message) => error!("[ Panic ]{}", check_message(message)),
        }
    }
}

pub mod led {
    use crate::status;
    use embassy_executor::task;

    #[cfg(feature = "smart-led")]
    use esp_hal::{gpio::AnyPin, rmt::ChannelCreator, Async};

    #[cfg(feature = "smart-led")]
    #[task]
    pub async fn led_run(
        rmt_channel: ChannelCreator<Async, 0>,
        gpio_pin: AnyPin<'static>,
        led_brightness: u8,
    ) {
        use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
        use smart_leds::{brightness, colors, SmartLedsWriteAsync, RGB8};

        let mut status = status::STATUS.subscriber().unwrap();

        let mut led =
            SmartLedsAdapterAsync::new(rmt_channel, gpio_pin, [0u32; buffer_size_async(1)]);

        loop {
            let curent_status = status.next_message_pure().await;

            let color: RGB8 = match curent_status {
                status::Status::Ok(_) => colors::WHITE_SMOKE,
                status::Status::Warn(_) => colors::YELLOW,
                status::Status::Err(_) => colors::DARK_RED,
            };

            let _ = led
                .write(brightness([color].into_iter(), led_brightness))
                .await;
        }
    }
}
