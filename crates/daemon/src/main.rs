use hidapi::HidApi;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

const QS_SOCKET_PATH: &str = "/tmp/crab.sock";

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "value")]
enum HudEvent {
    Button(bool),
    Scroll(i8),
}

#[derive(Serialize, Debug)]
struct HudPayload {
    action: &'static str,
    event: HudEvent,
}

/**
 *
 * This will essentially look like
 * {"action": "example", "event": { "type": "Button", "value": true }}
 *
 */

fn main() {
    println!("Starting Logi Bolt Sniffer...");
    let api = HidApi::new().expect("Failed to init HID API");

    let mut bolt_interfaces = vec![];

    for device_info in api.device_list() {
        if device_info.vendor_id() == 0x046d && device_info.product_id() == 0xc548 {
            let interface = device_info.interface_number();
            println!("Found Bolt on interface {}", interface);

            if let Ok(device) = device_info.open_device(&api) {
                device.set_blocking_mode(false).unwrap();
                bolt_interfaces.push((interface, device));
            } else {
                println!("Failed to open interface {}", interface);
            }
        }
    }

    if bolt_interfaces.is_empty() {
        println!("No Logi Bolt devices found. Check connection!");
        return;
    }

    println!(
        "Listening on {} interfaces. Press the gesture button!",
        bolt_interfaces.len()
    );

    let mut buf = [0u8; 64];

    loop {
        for (interface, device) in &bolt_interfaces {
            match device.read(&mut buf) {
                Ok(res) if res > 0 => {
                    let packet = &buf[..res];
                    let hex_string: Vec<String> =
                        packet.iter().map(|b| format!("{:02X}", b)).collect();
                    println!("Interface [{}]: [{}]", interface, hex_string.join(", "));
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }
}
