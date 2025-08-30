#![no_std]
#![feature(impl_trait_in_assoc_type)]

// pub mod status;
pub mod web_server;
pub mod web_server_2;
pub mod wifi;

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

// struct LedIndecator<const BUFFER_SIZE: usize> {
//     brightness_level: u8,
//     colour: RGB8,
// }
//
// impl<const BUFFER_SIZE: usize> LedIndecator<{ BUFFER_SIZE }> {
//     pub async fn new(led_gpio: AnyPin<'static>, rmt_peripheral: peripherals::RMT<'static>, frequency: Rate) -> Self {
//         todo!()
//     }
// }
//
// #[task]
// pub async fn run(_led: &'static mut LedIndecator<0>) {
//     todo!()
// }
