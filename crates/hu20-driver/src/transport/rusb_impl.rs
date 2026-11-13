use super::{TransportError, UsbTransport};
use crate::protocol::{
    READ_SERIAL_REQUEST, RESET_DEVICE_REQUEST, SET_LED_REQUEST, START_IMAGE_REQUEST,
    START_SMART_IMAGE_REQUEST,
};
use secugen_hu20_rs as native;
use std::time::Duration;

/// Production USB transport utilizing native `secugen-hu20-rs` driver communication.
pub struct RusbTransport {
    device: native::SecuGenDevice,
    pending_buffer: Vec<u8>,
    buffer_offset: usize,
}

impl RusbTransport {
    /// Opens the first detected physical SecuGen HU20 scanner (`1162:2200`).
    pub fn open_first() -> Result<Self, native::ScannerError> {
        let device = native::SecuGenDevice::open_first()?;
        Ok(Self {
            device,
            pending_buffer: Vec::new(),
            buffer_offset: 0,
        })
    }

    /// Opens a specific physical SecuGen HU20 scanner by bus and address ID.
    pub fn open_by_id(id: &native::DeviceId) -> Result<Self, native::ScannerError> {
        let device = native::SecuGenDevice::open_by_id(id)?;
        Ok(Self {
            device,
            pending_buffer: Vec::new(),
            buffer_offset: 0,
        })
    }

    /// Access the underlying native device info.
    pub fn info(&self) -> &native::DeviceInfo {
        self.device.info()
    }

    /// Direct native frame capture (`300x400` resampled from `1100x800` `SensorFrame`).
    pub fn capture_frame(&mut self) -> Result<native::RawFrame, native::ScannerError> {
        self.device.capture_frame()
    }

    /// Direct native smart capture with closed-loop auto-exposure and gain tuning.
    pub fn capture_smart_frame(&mut self) -> Result<native::RawFrame, native::ScannerError> {
        self.device.capture_smart_frame()
    }

    /// Sets CMOS sensor exposure time (`500..=8000`).
    pub fn set_exposure(&mut self, exposure: u16) -> Result<(), native::ScannerError> {
        self.device.set_exposure(exposure)
    }

    /// Sets CMOS sensor analog gain (`0x04..=0x28`).
    pub fn set_gain(&mut self, gain: u8) -> Result<(), native::ScannerError> {
        self.device.set_gain(gain)
    }
}

impl UsbTransport for RusbTransport {
    fn write_bulk(
        &mut self,
        _endpoint: u8,
        data: &[u8],
        _timeout: Duration,
    ) -> Result<usize, TransportError> {
        if let Some(&opcode) = data.first() {
            match opcode {
                START_SMART_IMAGE_REQUEST => {
                    let frame = self
                        .device
                        .capture_smart_frame()
                        .map_err(|e| TransportError::Usb(e.to_string()))?;
                    self.pending_buffer = frame.into_bytes();
                    self.buffer_offset = 0;
                }
                START_IMAGE_REQUEST => {
                    let frame = self
                        .device
                        .capture_frame()
                        .map_err(|e| TransportError::Usb(e.to_string()))?;
                    self.pending_buffer = frame.into_bytes();
                    self.buffer_offset = 0;
                }
                _ => {}
            }
        }
        Ok(data.len())
    }

    fn read_bulk(
        &mut self,
        _endpoint: u8,
        buffer: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, TransportError> {
        if self.buffer_offset >= self.pending_buffer.len() {
            return Ok(0);
        }

        let remaining = self.pending_buffer.len() - self.buffer_offset;
        let to_copy = remaining.min(buffer.len());
        buffer[..to_copy].copy_from_slice(
            &self.pending_buffer[self.buffer_offset..self.buffer_offset + to_copy],
        );
        self.buffer_offset += to_copy;
        Ok(to_copy)
    }

    fn control_transfer(
        &mut self,
        _request_type: u8,
        request: u8,
        value: u16,
        _index: u16,
        data: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, TransportError> {
        match request {
            SET_LED_REQUEST => {
                self.device
                    .set_led(value != 0)
                    .map_err(|e| TransportError::Usb(e.to_string()))?;
                Ok(0)
            }
            RESET_DEVICE_REQUEST => {
                self.device
                    .reset()
                    .map_err(|e| TransportError::Usb(e.to_string()))?;
                Ok(0)
            }
            READ_SERIAL_REQUEST => {
                let serial = self
                    .device
                    .read_serial_number()
                    .map_err(|e| TransportError::Usb(e.to_string()))?;
                let to_copy = data.len().min(serial.len());
                data[..to_copy].copy_from_slice(&serial[..to_copy]);
                Ok(to_copy)
            }
            _ => Ok(0),
        }
    }
}
