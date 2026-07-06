use hid_protocol::device::{MouseResponse, deserialize};
use hidapi::HidApi;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread::sleep;
use std::time::Duration;

const QS_SOCKET_PATH: &str = "/tmp/crab.sock";

#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "value")]
enum HudEvent {
    Button(bool),
    Scroll(i8),
    Battery(u8),
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

    let _ = fs::remove_file(QS_SOCKET_PATH);
    let qs_socket = UnixListener::bind(QS_SOCKET_PATH).expect("Could not bind!");
    qs_socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let mut active_socket: Option<UnixStream> = None;

    loop {
        for (interface, device) in &bolt_interfaces {
            match device.read(&mut buf) {
                Ok(res) if res > 0 => {
                    let packet = &buf[..res];
                    let hex_string: Vec<String> =
                        packet.iter().map(|b| format!("{:02X}", b)).collect();
                    println!("Interface [{}]: [{}]", interface, hex_string.join(", "));
                    match deserialize(packet) {
                        Ok(mouse_resp) => {
                            let (action, event_variant) = match mouse_resp {
                                MouseResponse::GestureButton(val) => {
                                    ("gesture", HudEvent::Button(val))
                                }
                                MouseResponse::BatteryLevel(val) => {
                                    ("battery", HudEvent::Battery(val))
                                }
                                MouseResponse::ActionButton(val) => {
                                    ("action", HudEvent::Button(val))
                                }
                                MouseResponse::ForwardButton(val) => {
                                    ("forward", HudEvent::Button(val))
                                }
                                MouseResponse::BackButton(val) => ("back", HudEvent::Button(val)),
                                MouseResponse::HorizontalScroll(val) => {
                                    ("horizontal_scroll", HudEvent::Scroll(val))
                                }
                                MouseResponse::VerticalScroll(val) => {
                                    ("Vertical_scroll", HudEvent::Scroll(val))
                                }
                            };

                            let payload = HudPayload {
                                action: action,
                                event: event_variant,
                            };

                            match qs_socket.accept() {
                                Ok((sock, _)) => {
                                    active_socket = Some(sock);
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                Err(e) => println!("Connection failed: {}", e),
                            }

                            if let Some(sock) = &mut active_socket {
                                if let Ok(json_str) = serde_json::to_string(&payload) {
                                    let msg = format!("{}\n", json_str);
                                    let _ = sock.write_all(msg.as_bytes());
                                }
                            }
                        }

                        Err(_) => {
                            // TODO: ill think about it
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }
}
