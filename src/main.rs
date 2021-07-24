extern crate serial;
extern crate min_rs as min;

use std::time::Duration;
use std::thread;
use serial::prelude::*;
use serial::SystemPort;
use std::io::prelude::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use log::{LevelFilter, info, debug, trace};
use env_logger;

struct App {
    name: String,
}

impl min::Name for App {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl App {
    fn new(name: String) -> Self {
        App{
            name: name,
        }
    }
    fn print_msg(&self, buffer: &[u8], len: u8) {
        let mut output = String::from("");
        output.push_str(format!("receive data: [ ").as_str());
        for i in 0..len {
            output.push_str(format!("0x{:02x} ", buffer[i as usize]).as_str());
        }
        output.push_str(format!("]").as_str());
        info!(target: self.name.as_str(), "{}", output);
    }
}

struct Uart {
    port: RefCell<SystemPort>,
    name: String,
    tx_space_avaliable: u16,
    output: Arc<Mutex<String>>,
}

impl Uart {
    fn new(port: SystemPort, name: String, tx_space_avaliable: u16) -> Self {
        Uart{
            port: RefCell::new(port),
            name: name,
            tx_space_avaliable: tx_space_avaliable,
            output: Arc::new(Mutex::new(String::from(""))),
        }
    }

    fn open(&self) {
        const SETTINGS: serial::PortSettings = serial::PortSettings {
            baud_rate: serial::Baud115200,
            char_size: serial::Bits8,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
            flow_control: serial::FlowNone,
        };
        let mut port = self.port.borrow_mut();
        port.configure(&SETTINGS).unwrap();
        port.set_timeout(Duration::from_millis(1000)).unwrap();
        debug!(target: self.name.as_str(), "{}: Open uart.", self.name);
    }

    fn available_for_write(&self) -> u16 {
        self.tx_space_avaliable
    }

    fn tx(&self, byte: u8) {
        let mut output = self.output.lock().unwrap();
        output.push_str(format!("0x{:02x} ", byte).as_str());
        let mut port = self.port.borrow_mut();
        match port.write(&[byte]) {
            Ok(_) => {},
            Err(e) => {
                debug!(target: self.name.as_str(), "{}", e);
            },
        }
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut port = self.port.borrow_mut();
        match port.read(&mut buf[..]) {
            Ok(n) => Ok(n),
            _ => Err(()),
        }
    }
}

fn tx_start(uart: &Uart) {
    let mut output = uart.output.lock().unwrap();
    output.clear();
    output.push_str(format!("send frame: [ ").as_str());
}

fn tx_finished(uart: &Uart) {
    let mut output = uart.output.lock().unwrap();
    output.push_str(format!("]").as_str());
    trace!(target: uart.name.as_str(), "{}", output);
}
fn tx_space(uart: &Uart) -> u16 {
    uart.available_for_write()
}

fn tx_byte(uart: &Uart, _min_port: u8, byte: u8) {
    uart.tx(byte);
}

fn rx_byte(min: &mut min::Context<Uart, App>, buf: &[u8], buf_len: u32) {
    min.poll(buf, buf_len);
}

fn application_handler(app: &App, _min_id: u8, buffer: &[u8], len: u8, _port: u8) {
    app.print_msg(buffer, len);
}

fn main() {
    log::set_max_level(LevelFilter::Debug);
    env_logger::init();
    let tx_data: [u8; 3] = [1, 2, 3];
    let port = serial::open("/dev/ttyS9").unwrap();
    let uart = Uart::new(port, String::from("uart"), 128);
    let app = App::new(String::from("app"));
    let mut min = min::Context::new(
        &uart,
        &app,
        0,
        true,
        tx_start,
        tx_finished,
        tx_space,
        tx_byte,
        application_handler,
    );
    min.hw_if.open();

    let mut buf: Vec<u8> = (0..255).collect();
    min.reset_transport(true).unwrap_or(());
    min.queue_frame(0, &tx_data[..], tx_data.len() as u8).unwrap_or(());
    loop {
        min.poll(&[0][0..0], 0);
        if let Ok(n) = min.hw_if.read(&mut buf[..]) {
            rx_byte(&mut min, &buf[0..n], n as u32);
        };
        thread::sleep(Duration::from_millis(10));
    }
}