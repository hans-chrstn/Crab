pub trait HidDevice {
    fn write(&mut self, data: &[u8]) -> Result<usize, String>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, String>;
}

pub fn parse_battery<D: HidDevice>(device: &mut D) -> Result<u8, String> {
    let mut buf = [0u8; 7];
    device.read(&mut buf)?;
    Ok(buf[4])
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
}
