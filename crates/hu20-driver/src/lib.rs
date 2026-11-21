pub mod frame;
pub mod protocol;
pub mod transport;

pub use frame::{FrameAssembler, FrameError, parse_raw_frame};
pub use protocol::{
    BULK_IN_ENDPOINT, CommandOpcode, HU20_PID, ProtocolError, ResponsePacket, SECUGEN_VID,
    UsbCommand, parse_response_packet,
};
pub use transport::{MockScenario, MockTransport, RusbTransport, TransportError, UsbTransport};

use hu20_core::{DeviceInfo, FingerprintScanner, RawFrame};
use secugen_hu20_rs as native;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error(transparent)]
    Native(#[from] native::ScannerError),

    #[error(transparent)]
    Core(#[from] hu20_core::CoreError),

    #[error(transparent)]
    Transport(#[from] TransportError),

    #[error(transparent)]
    Frame(#[from] frame::FrameError),

    #[error(transparent)]
    Protocol(#[from] protocol::ProtocolError),

    #[error("timed out waiting for finger on scanner after {0:?}")]
    Timeout(Duration),
}

fn device_info(value: &native::DeviceInfo) -> DeviceInfo {
    DeviceInfo {
        vendor_id: value.vendor_id,
        product_id: value.product_id,
        bus_number: value.bus_number,
        address: value.address,
        model: 20,
    }
}

/// Discovers connected physical SecuGen HU20 scanners (`1162:2200`) on the USB bus,
/// or reports simulated devices if `HU20_MOCK_FIXTURE` is configured.
pub fn list_devices() -> Result<Vec<DeviceInfo>, DriverError> {
    if let Ok(scenario) = std::env::var("HU20_MOCK_FIXTURE").and_then(|v| {
        v.parse::<MockScenario>()
            .map_err(|_| std::env::VarError::NotPresent)
    }) {
        return list_devices_with_mock(Some(scenario));
    }
    list_devices_with_mock(None)
}

/// Discovers devices with explicit mock scenario control.
pub fn list_devices_with_mock(mock: Option<MockScenario>) -> Result<Vec<DeviceInfo>, DriverError> {
    if let Some(scenario) = mock {
        return match scenario {
            MockScenario::Disconnect => Ok(Vec::new()),
            _ => Ok(vec![DeviceInfo {
                vendor_id: SECUGEN_VID,
                product_id: HU20_PID,
                bus_number: 1,
                address: 2,
                model: 20,
            }]),
        };
    }

    Ok(native::list_devices()?
        .into_iter()
        .map(|d| DeviceInfo {
            vendor_id: d.vendor_id,
            product_id: d.product_id,
            bus_number: d.id.bus_number,
            address: d.id.address,
            model: 20,
        })
        .collect())
}

/// Type-safe biometric scanner device parameterized by its USB transport abstraction.
pub struct Device<T: UsbTransport> {
    transport: T,
    info: DeviceInfo,
}

impl Device<RusbTransport> {
    /// Opens the first available physical SecuGen HU20 scanner (`1162:2200`) on the USB bus.
    pub fn open_first() -> Result<Self, DriverError> {
        let transport = RusbTransport::open_first()?;
        let info = device_info(transport.info());
        Ok(Self { transport, info })
    }

    /// Opens a specific physical SecuGen HU20 scanner identified by bus and address.
    pub fn open(info: DeviceInfo) -> Result<Self, DriverError> {
        let id = native::DeviceId::new(info.bus_number, info.address);
        let transport = RusbTransport::open_by_id(&id)?;
        Ok(Self { transport, info })
    }
}

impl Device<MockTransport> {
    /// Instantiates a virtual mock scanner device for automated testing.
    pub fn mock(transport: MockTransport) -> Self {
        let info = DeviceInfo {
            vendor_id: SECUGEN_VID,
            product_id: HU20_PID,
            bus_number: 1,
            address: 2,
            model: 20,
        };
        Self { transport, info }
    }
}

impl<T: UsbTransport + 'static> Device<T> {
    /// Converts a concrete generic device into a dynamic trait-object boxed device.
    pub fn boxed(self) -> Device<Box<dyn UsbTransport>> {
        Device {
            transport: Box::new(self.transport),
            info: self.info,
        }
    }
}

impl<T: UsbTransport> Device<T> {
    /// Constructs a device from any transport and device metadata.
    pub fn new(transport: T, info: DeviceInfo) -> Self {
        Self { transport, info }
    }

    /// Returns device metadata.
    pub fn device_info(&self) -> Option<DeviceInfo> {
        Some(self.info.clone())
    }

    /// Reference to underlying transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Mutable reference to underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Captures a biometric frame using standard or closed-loop auto-exposure (`smart = true`).
    pub fn capture_with_mode(&mut self, smart: bool) -> Result<RawFrame, DriverError> {
        let opcode = if smart {
            CommandOpcode::StartSmartImage
        } else {
            CommandOpcode::StartImage
        };
        let cmd = UsbCommand::new(opcode, None);
        self.transport
            .write_bulk(0x01, &cmd.to_bytes(), Duration::from_millis(2500))?;

        let mut assembler = FrameAssembler::new();
        let chunk_timeout = Duration::from_millis(1500);
        let mut chunk_buf = [0u8; 4096];

        while !assembler.is_complete() {
            let n = self
                .transport
                .read_bulk(BULK_IN_ENDPOINT, &mut chunk_buf, chunk_timeout)?;
            if n == 0 {
                break;
            }
            if let Some(valid_slice) = chunk_buf.get(..n) {
                assembler.append_chunk(valid_slice)?;
            }
        }

        Ok(assembler.finalize()?)
    }

    /// Captures a biometric frame using closed-loop auto-exposure.
    pub fn capture_smart(&mut self) -> Result<RawFrame, DriverError> {
        self.capture_with_mode(true)
    }

    /// Waits for finger presence on the sensor surface with contact stabilization.
    pub fn wait_for_finger<F>(
        &mut self,
        timeout: Duration,
        min_quality: u8,
        smart: bool,
        on_poll: F,
    ) -> Result<RawFrame, DriverError>
    where
        F: FnMut(&hu20_core::FingerPresence, Duration),
    {
        self.wait_for_finger_with_options(
            timeout,
            min_quality,
            smart,
            WaitOptions::default(),
            on_poll,
        )
    }

    /// Wait using an explicit polling interval.
    pub fn wait_for_finger_with_options<F>(
        &mut self,
        timeout: Duration,
        min_quality: u8,
        smart: bool,
        options: WaitOptions,
        mut on_poll: F,
    ) -> Result<RawFrame, DriverError>
    where
        F: FnMut(&hu20_core::FingerPresence, Duration),
    {
        let start = Instant::now();
        let poll_interval = options.poll_interval;

        while start.elapsed() < timeout {
            let frame = self.capture_with_mode(smart)?;
            let presence = hu20_core::evaluate_presence(&frame);
            on_poll(&presence, start.elapsed());

            if presence.is_present && presence.quality.score >= min_quality {
                return Ok(frame);
            }
            std::thread::sleep(poll_interval.min(timeout.saturating_sub(start.elapsed())));
        }

        Err(DriverError::Timeout(timeout))
    }
}

impl<T: UsbTransport> FingerprintScanner for Device<T> {
    type Error = DriverError;

    fn capture(&mut self) -> Result<RawFrame, Self::Error> {
        self.capture_with_mode(true)
    }
}

/// Dynamic trait-object scanner compatible with both real physical USB hardware
/// and simulated virtual mock fixtures.
pub type Hu20Scanner = Device<Box<dyn UsbTransport>>;

impl Hu20Scanner {
    /// Opens the first available physical USB scanner.
    pub fn open_first() -> Result<Self, DriverError> {
        Ok(Device::<RusbTransport>::open_first()?.boxed())
    }

    /// Opens a specific physical USB scanner.
    pub fn open(info: DeviceInfo) -> Result<Self, DriverError> {
        Ok(Device::<RusbTransport>::open(info)?.boxed())
    }

    /// Creates a virtual mock scanner device for automated testing.
    pub fn new_mock(transport: MockTransport) -> Self {
        Device::<MockTransport>::mock(transport).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hu20_core::FRAME_LEN;

    #[test]
    fn test_mock_device_capture_match() {
        let mock = MockTransport::new(MockScenario::Match);
        let mut device = Device::mock(mock);
        let frame = device.capture().expect("capture match");
        assert_eq!(frame.pixels().len(), FRAME_LEN);
        let quality = hu20_core::evaluate_quality(&frame);
        assert!(quality.score > 70);
    }

    #[test]
    fn test_mock_device_capture_disconnect() {
        let mock = MockTransport::new(MockScenario::Disconnect);
        let mut device = Device::mock(mock);
        let result = device.capture();
        assert!(result.is_err());
        match result.unwrap_err() {
            DriverError::Transport(TransportError::DeviceDisconnected) => {}
            other => panic!("expected DeviceDisconnected, got: {other:?}"),
        }
    }

    #[test]
    fn test_mock_device_wait_for_finger_match() {
        let mock = MockTransport::new(MockScenario::Match);
        let mut device = Device::mock(mock);
        let mut poll_count = 0;
        let frame = device
            .wait_for_finger(Duration::from_millis(500), 50, true, |_, _| {
                poll_count += 1;
            })
            .expect("wait for finger should succeed immediately for match scenario");
        assert!(poll_count >= 1);
        assert_eq!(frame.pixels().len(), FRAME_LEN);
    }

    #[test]
    fn test_mock_device_wait_for_finger_empty_timeout() {
        let mock = MockTransport::new(MockScenario::Empty);
        let mut device = Device::mock(mock);
        let result = device.wait_for_finger(Duration::from_millis(100), 50, true, |_, _| {});
        assert!(result.is_err());
        match result.unwrap_err() {
            DriverError::Timeout(_) => {}
            other => panic!("expected Timeout, got: {other:?}"),
        }
    }
}

/// Operational settings for finger-presence polling.
#[derive(Clone, Copy, Debug)]
pub struct WaitOptions {
    pub poll_interval: Duration,
}
impl Default for WaitOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(150),
        }
    }
}
