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
}

// TODO: deserialize, use in parse_battery
// pub fn deserialize() -> Result<MouseResponse, String> {
// }

pub trait HidDevice {
    fn write(&mut self, data: &[u8]) -> Result<usize, String>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, String>;
}

pub fn parse_battery<D: HidDevice>(device: &mut D) -> Result<u8, String> {
    let mut buf = [0u8; 7];
    let res = device.read(&mut buf)?;
    if res < 7 {
        return Err("Packet too short".to_string());
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
        fn write(&mut self, _data: &[u8]) -> Result<usize, String> {
            Ok(0)
        }

        fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
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
        assert!(res.is_err())
    }

    #[test]
    fn test_build_set_dpi_command() {
        let res = build_set_dpi_command(1000);
        assert_eq!(res[4], 0x03);
        assert_eq!(res[5], 0xE8);
    }
}
