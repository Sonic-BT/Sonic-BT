use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use log::info;

pub enum LEDStatus {
    Ok(),
    Progressing(),
    Panic(),
}

pub static LED_STATUS: Signal<CriticalSectionRawMutex, LEDStatus> = Signal::new();

#[embassy_executor::task]
pub async fn run() -> () {
    loop {
        let led_status: LEDStatus = LED_STATUS.wait().await;

        match led_status {
            LEDStatus::Ok() => info!("Ok"),
            LEDStatus::Progressing() => info!("Progressing"),
            LEDStatus::Panic() => info!("Panic"),
        }
    }
}
