// #![feature(portable_simd)]
#![allow(dead_code, unused_imports)]
#[path = "interfaces/uci.rs"] mod uci;

use std::{io};
// use uci::UCIInterface;

fn process_ui_commands(uci_interface: &mut uci::UCIInterface) {
    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_n_bytes) => {
                let cmd = buffer.as_str().trim();
                if !uci_interface.process_command(&cmd) { break; }

                // println!("{} bytes read", n);
                // println!("{}", buffer);
            }
            Err(e) => println!("Error while reading from stdin: {}", e),
        }
    }
}

fn main() {
    let mut uci = uci::UCIInterface::init_interface();
    process_ui_commands(&mut uci);
}
