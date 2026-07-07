use crate::error::HidError;

pub enum MouseCommand {
    SetDpi(u16),
    GetBattery,
    SetActionsButton,
    SetScrollWheelAction,
    SetGesture,
    SetBackButton,
    SetForwardButton,
}

pub fn serialize(command: MouseCommand) -> [u8; 7] {
    match command {
        MouseCommand::SetDpi(val) => {
            let high = (val >> 8) as u8;
            let low = val as u8;
            [0x10, 0x00, 0x00, 0x00, high, low, 0x00]
        }

        MouseCommand::GetBattery => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        MouseCommand::SetScrollWheelAction => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        MouseCommand::SetGesture => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        MouseCommand::SetActionsButton => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        MouseCommand::SetBackButton => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        MouseCommand::SetForwardButton => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}

pub enum MouseResponse {
    BatteryLevel(u8),
    GestureButton(bool),
    ActionButton(bool),
    ForwardButton(bool),
    BackButton(bool),
    MiddleClick(bool),
    HorizontalScroll(i8),
    VerticalScroll(i8),
}

pub fn deserialize(data: &[u8]) -> Result<MouseResponse, HidError> {
    if data.len() < 2 {
        return Err(HidError::PacketTooShort {
            expected: 2,
            got: data.len(),
        });
    }

    match data[0] {
        0x02 => {
            if data.len() < 9 {
                return Err(HidError::PacketTooShort {
                    expected: 9,
                    got: data.len(),
                });
            }

            if data[1] == 0x40 {
                return Ok(MouseResponse::GestureButton(true));
            }
            if data[1] == 0x04 {
                return Ok(MouseResponse::MiddleClick(true));
            }
            if data[1] == 0x20 {
                return Ok(MouseResponse::ActionButton(true));
            }

            if data[3] == 0x04 {
                return Ok(MouseResponse::ForwardButton(true));
            }
            if data[3] == 0x02 {
                return Ok(MouseResponse::BackButton(true));
            }

            let vert = data[7] as i8;
            if vert != 0 {
                return Ok(MouseResponse::VerticalScroll(vert));
            }

            let horiz = data[8] as i8;
            if horiz != 0 {
                return Ok(MouseResponse::HorizontalScroll(horiz));
            }

            if data[1] == 0x00 && data[3] == 0x00 && data[7] == 0x00 && data[8] == 0x00 {
                return Ok(MouseResponse::GestureButton(false));
            }

            Err(HidError::InvalidHeader(data[1]))
        }
        0x11 => {
            if data.len() < 5 {
                return Err(HidError::PacketTooShort {
                    expected: 5,
                    got: data.len(),
                });
            }

            Ok(MouseResponse::BatteryLevel(data[4]))
        }
        _ => Err(HidError::InvalidHeader(data[0])),
    }
}

pub trait HidDevice {
    fn write(&mut self, data: &[u8]) -> Result<usize, HidError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, HidError>;
}

pub fn parse_battery<D: HidDevice>(device: &mut D) -> Result<u8, HidError> {
    let mut buf = [0u8; 7];
    let res = device.read(&mut buf)?;
    if res < buf.len() {
        return Err(HidError::PacketTooShort {
            expected: buf.len(),
            got: res,
        });
    }
    Ok(buf[4])
}

pub fn build_set_dpi_command(dpi: u16) -> [u8; 7] {
    return serialize(MouseCommand::SetDpi(dpi));
}

#[cfg(test)]
mod tests {
    use super::*;
    pub struct MockMouse {
        mock_data: Vec<u8>,
    }

    impl HidDevice for MockMouse {
        fn write(&mut self, _data: &[u8]) -> Result<usize, HidError> {
            Ok(0)
        }

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, HidError> {
            let len = self.mock_data.len();
            // check all slices from start to len: buf[..len]
            buf[..len].copy_from_slice(&self.mock_data);
            Ok(len)
        }
    }

    #[test]
    fn test_parse_battery() {
        let test_packet = vec![0x11, 0x01, 0x00, 0x10, 55, 0x00, 0x00];
        let mut device = MockMouse {
            mock_data: test_packet,
        };
        let battery = parse_battery(&mut device).unwrap();
        assert_eq!(battery, 55)
    }

    #[test]
    fn test_parse_battery_short_packet() {
        let test_packet = vec![0x11, 0x01, 0x00];
        let mut device = MockMouse {
            mock_data: test_packet,
        };
        let res = parse_battery(&mut device);
        assert!(matches!(res, Err(HidError::PacketTooShort { .. })));
    }

    #[test]
    fn test_build_set_dpi_command() {
        let res = build_set_dpi_command(1000);
        assert_eq!(res[4], 0x03);
        assert_eq!(res[5], 0xE8);
    }

    #[test]
    fn test_deserialize() {
        let battery_packet = [0x11, 0x00, 0x00, 0x00, 80, 0x00, 0x00];
        let res = deserialize(&battery_packet).unwrap();
        assert!(matches!(res, MouseResponse::BatteryLevel(80)));

        let gesture_down = [0x02, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = deserialize(&gesture_down).unwrap();
        assert!(matches!(res, MouseResponse::GestureButton(true)));

        let action_down = [0x02, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = deserialize(&action_down).unwrap();
        assert!(matches!(res, MouseResponse::ActionButton(true)));

        let scroll_up = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00];
        let res = deserialize(&scroll_up).unwrap();
        assert!(matches!(res, MouseResponse::VerticalScroll(-1)));

        let thumb_right = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let res = deserialize(&thumb_right).unwrap();
        assert!(matches!(res, MouseResponse::HorizontalScroll(1)));

        let middle_click = [0x02, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = deserialize(&middle_click).unwrap();
        assert!(matches!(res, MouseResponse::MiddleClick(true)));

        let release_packet = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = deserialize(&release_packet).unwrap();
        assert!(matches!(res, MouseResponse::GestureButton(false)));

        let short_packet = [0x02, 0x00];
        let res = deserialize(&short_packet);
        assert!(matches!(res, Err(HidError::PacketTooShort { .. })));
    }
}
