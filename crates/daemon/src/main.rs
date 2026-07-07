use hid_protocol::device::{MouseResponse, deserialize};
use hidapi::HidApi;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread::sleep;
use std::time::Duration;

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

    let socket_path = if std::path::Path::new("/run/crab").exists() {
        "/run/crab/api.sock"
    } else {
        println!(
            "Warning: /run/crab does not exist (not running via systemd). Falling back to /tmp/crab.sock"
        );
        "/tmp/crab.sock"
    };

    let _ = fs::remove_file(socket_path);
    let qs_socket = UnixListener::bind(socket_path).unwrap_or_else(|e| {
        panic!("Could not bind to {}: {}", socket_path, e);
    });

    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666));

    qs_socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    let mut active_sockets: Vec<UnixStream> = Vec::new();

    loop {
        match qs_socket.accept() {
            Ok((sock, _)) => {
                println!("New client connected!");
                active_sockets.push(sock);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => println!("Connection failed: {}", e),
        }

        for (_interface, device) in &bolt_interfaces {
            match device.read(&mut buf) {
                Ok(res) if res > 0 => {
                    let packet = &buf[..res];
                    match deserialize(packet) {
                        Ok(mouse_resp) => {
                            let (action, event_variant) = match mouse_resp {
                                MouseResponse::GestureButton(val) => {
                                    ("gesture", HudEvent::Button(val))
                                }
                                MouseResponse::MiddleClick(val) => {
                                    ("middle_click", HudEvent::Button(val))
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

                            if let Ok(json_str) = serde_json::to_string(&payload) {
                                let msg = format!("{}\n", json_str);

                                active_sockets.retain_mut(|sock| {
                                    match sock.write_all(msg.as_bytes()) {
                                        Ok(_) => true,
                                        Err(_) => {
                                            println!("Client disconnected.");
                                            false
                                        }
                                    }
                                });
                            }
                        }
                        Err(_) => continue,
                    }
                }
                _ => {}
            }
        }
        sleep(Duration::from_millis(5));
    }
}
