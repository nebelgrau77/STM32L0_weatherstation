// Will only work with Range5 or higher
// Needs Range6 or higher to work with 400kHz I2C frequency

//#![deny(warnings)]
//#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate panic_halt;
extern crate shared_bus;


use cortex_m_rt::entry;
use stm32l0xx_hal::{pac, prelude::*, rcc::{Config,MSIRange}};

use bme280::BME280;
use ssd1306::{prelude::*, Builder as SSD1306Builder};

use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
    };

use core::fmt;
use core::fmt::Write;

use arrayvec::ArrayString;

const BOOT_DELAY_MS: u16 = 100; 

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Configure the clock.
    //let mut rcc = dp.RCC.freeze(Config::hsi16()); 
    let mut rcc = dp.RCC.freeze(Config::msi(MSIRange::Range5)); //works only with Range5 or Range6

    let mut delay = cp.SYST.delay(rcc.clocks);

    //delay necessary for the I2C to initiate correctly and start on boot without having to reset the board

    delay.delay_ms(BOOT_DELAY_MS);

    // Acquire the GPIOA peripheral. This also enables the clock for GPIOA in
    // the RCC register.
    let gpioa = dp.GPIOA.split(&mut rcc);

    let scl = gpioa.pa9.into_open_drain_output();
    let sda = gpioa.pa10.into_open_drain_output();
    
    let mut i2c = dp.I2C1.i2c(sda, scl, 100.khz(), &mut rcc); 

    let manager = shared_bus::CortexMBusManager::new(i2c);
    
    let bme280_i2c_addr = 0x76;
    let mut bme280 = BME280::new(manager.acquire(), bme280_i2c_addr, delay);    
    bme280.init().unwrap();    

    //let mut disp: GraphicsMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(manager.acquire()).into();
        
    let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.clear().unwrap();
    
    //let text_style = TextStyleBuilder::new(Font8x16).text_color(BinaryColor::On).build();

    
    loop {

        let measurements = bme280.measure().unwrap();
        let temp = measurements.temperature;
        let hum = measurements.humidity;

        //let mut t_buf = ArrayString::<[u8; 7]>::new();
        //let mut h_buf = ArrayString::<[u8; 7]>::new();
        let mut buffer = ArrayString::<[u8; 64]>::new();
     
        /*
        format(&mut t_buf, temp as u8, 67 as char, 84 as char); // 67 is "C" in ASCII, 84 is "T"
        Text::new(t_buf.as_str(), Point::new(16, 0)).into_styled(text_style).draw(&mut disp);
        
        format(&mut h_buf, hum as u8, 37 as char, 72 as char); // 37 is "%" in ASCII, 72 is "H"
        Text::new(h_buf.as_str(), Point::new(16, 16)).into_styled(text_style).draw(&mut disp);
        */

        format(&mut buffer, temp as u8, hum as u8);
            
        disp.write_str(buffer.as_str()).unwrap();
        
        //write_str(buffer.as_str());

        //disp.flush().unwrap();


        //cortex_m::asm::delay(1 * 4_000_000); //roughly every 2 seconds? WON'T WORK
    
    }

}

/*
fn format(buf: &mut ArrayString<[u8; 7]>, data: u8, unit: char, param: char) {
    fmt::write(buf, format_args!("{}: {:02} {}", param, data, unit)).unwrap();
}
*/

fn format(buf: &mut ArrayString<[u8; 64]>, temp: u8, hum: u8) {
    fmt::write(buf, format_args!("T: {:02} C                         H: {:02} %                         ", temp, hum)).unwrap();
}