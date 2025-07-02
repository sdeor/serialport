mod dcb;

use winapi::shared::minwindef::{BOOL, DWORD, LPVOID};

use winapi::um::handleapi::{self, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::winbase::RTS_CONTROL_DISABLE;
use winapi::um::winnt::{
    DUPLICATE_SAME_ACCESS, FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE, HANDLE, MAXDWORD,
};
use winapi::um::{commapi, fileapi, winbase};

use crate::builder::SerialPortBuilder;
use crate::config::{DataBits, FlowControl, Parity, StopBits};

fn winapi_result(result: BOOL) -> std::io::Result<()> {
    match result {
        0 => Err(std::io::Error::last_os_error()),
        _ => Ok(()),
    }
}

fn prefix_port(port: &str) -> String {
    static PORT_PREFIX: &str = r"\\.\";

    if !port.starts_with(PORT_PREFIX) {
        format!(r"\\.\{}", port)
    } else {
        port.to_string()
    }
}

pub(super) struct SerialPort {
    is_open: bool,
    handle: HANDLE,
    builder: SerialPortBuilder,
}

unsafe impl Send for SerialPort {}

impl SerialPort {
    pub fn available_ports() -> std::io::Result<Vec<String>> {
        // TODO: Implement
        unimplemented!()
    }

    pub fn new(builder: &SerialPortBuilder) -> std::io::Result<Self> {
        let mut serialport = Self {
            is_open: false,
            handle: INVALID_HANDLE_VALUE,
            builder: builder.clone(),
        };

        if !serialport.builder.port.is_empty() {
            serialport.open()?;
        }

        Ok(serialport)
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        let process = unsafe { GetCurrentProcess() };
        let mut handle = INVALID_HANDLE_VALUE;

        winapi_result(unsafe {
            handleapi::DuplicateHandle(
                process,
                self.handle,
                process,
                &mut handle,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        })?;

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        Ok(SerialPort {
            is_open: self.is_open,
            handle,
            builder: self.builder.clone(),
        })
    }

    fn reconfigure(&mut self) -> std::io::Result<()> {
        if self.handle == INVALID_HANDLE_VALUE {
            return Err(std::io::ErrorKind::NotConnected.into());
        }

        let result = dcb::WindowsDCB::get(self.handle)?
            .update(&self.builder)
            .set(self.handle);

        match result {
            Err(e) => {
                let _ = self.close();
                return Err(e);
            }
            Ok(_) => (),
        };

        let milliseconds =
            u128::min(self.builder.timeout.as_millis(), MAXDWORD as u128 - 1) as DWORD;

        let mut timeouts = winbase::COMMTIMEOUTS {
            ReadIntervalTimeout: MAXDWORD,
            ReadTotalTimeoutMultiplier: 0,
            ReadTotalTimeoutConstant: milliseconds,
            WriteTotalTimeoutMultiplier: 0,
            WriteTotalTimeoutConstant: milliseconds,
        };

        winapi_result(unsafe { commapi::SetCommTimeouts(self.handle, &mut timeouts) })
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) -> std::io::Result<()> {
        if self.builder.port.is_empty() {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        if self.is_open {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }

        let path = prefix_port(
            self.builder
                .port
                .to_str()
                .ok_or(std::io::ErrorKind::InvalidInput)?,
        );
        let mut name = Vec::<u16>::with_capacity(path.len() + 1);
        name.extend(path.encode_utf16());
        name.push(0);

        let handle = unsafe {
            fileapi::CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null_mut(),
                fileapi::OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        self.handle = handle;

        self.reconfigure()?;
        self.is_open = true;

        Ok(())
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        if !self.is_open {
            return Ok(());
        }

        if self.handle != INVALID_HANDLE_VALUE {
            winapi_result(unsafe { handleapi::CloseHandle(self.handle) })?;
            self.handle = INVALID_HANDLE_VALUE;
        }

        self.is_open = false;

        Ok(())
    }

    pub fn port(&self) -> &str {
        self.builder.port.to_str().unwrap()
    }

    pub fn baud_rate(&self) -> std::io::Result<u32> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        Ok(dcb.inner.BaudRate)
    }

    pub fn data_bits(&self) -> std::io::Result<DataBits> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        match dcb.inner.ByteSize {
            5 => Ok(DataBits::Five),
            6 => Ok(DataBits::Six),
            7 => Ok(DataBits::Seven),
            8 => Ok(DataBits::Eight),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn flow_control(&self) -> std::io::Result<FlowControl> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        if dcb.inner.fOutxCtsFlow() != 0 || dcb.inner.fRtsControl() != RTS_CONTROL_DISABLE {
            Ok(FlowControl::Hardware)
        } else if dcb.inner.fOutX() != 0 || dcb.inner.fInX() != 0 {
            Ok(FlowControl::Software)
        } else {
            Ok(FlowControl::None)
        }
    }

    pub fn parity(&self) -> std::io::Result<Parity> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        match dcb.inner.Parity {
            winbase::NOPARITY => Ok(Parity::None),
            winbase::ODDPARITY => Ok(Parity::Odd),
            winbase::EVENPARITY => Ok(Parity::Even),
            winbase::MARKPARITY => Ok(Parity::Mark),
            winbase::SPACEPARITY => Ok(Parity::Space),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn stop_bits(&self) -> std::io::Result<StopBits> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        match dcb.inner.StopBits {
            winbase::ONESTOPBIT => Ok(StopBits::One),
            winbase::ONE5STOPBITS => Ok(StopBits::OnePointFive),
            winbase::TWOSTOPBITS => Ok(StopBits::Two),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    pub fn timeout(&self) -> std::time::Duration {
        self.builder.timeout
    }

    pub fn set_port<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.builder.port = path.as_ref().as_os_str().into();
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) -> std::io::Result<()> {
        self.builder.baud_rate = baud_rate;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    pub fn set_data_bits(&mut self, data_bits: DataBits) -> std::io::Result<()> {
        self.builder.data_bits = data_bits;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    pub fn set_flow_control(&mut self, flow_control: FlowControl) -> std::io::Result<()> {
        self.builder.flow_control = flow_control;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    pub fn set_parity(&mut self, parity: Parity) -> std::io::Result<()> {
        self.builder.parity = parity;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    pub fn set_stop_bits(&mut self, stop_bits: StopBits) -> std::io::Result<()> {
        self.builder.stop_bits = stop_bits;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    pub fn set_timeout(&mut self, timeout: std::time::Duration) -> std::io::Result<()> {
        self.builder.timeout = timeout;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }
}

impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self.is_open {
            return Err(std::io::ErrorKind::NotConnected.into());
        }

        let mut bytes_read: DWORD = 0;

        winapi_result(unsafe {
            fileapi::ReadFile(
                self.handle,
                buf.as_mut_ptr() as LPVOID,
                buf.len() as DWORD,
                &mut bytes_read,
                std::ptr::null_mut(),
            )
        })?;

        match bytes_read {
            0 => Err(std::io::ErrorKind::TimedOut.into()),
            _ => Ok(bytes_read as usize),
        }
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.is_open {
            return Err(std::io::ErrorKind::NotConnected.into());
        }

        let mut bytes_written: DWORD = 0;

        winapi_result(unsafe {
            fileapi::WriteFile(
                self.handle,
                buf.as_ptr() as LPVOID,
                buf.len() as DWORD,
                &mut bytes_written,
                std::ptr::null_mut(),
            )
        })?;

        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.is_open {
            return Err(std::io::ErrorKind::NotConnected.into());
        }

        winapi_result(unsafe { fileapi::FlushFileBuffers(self.handle) })?;

        Ok(())
    }
}

impl Drop for SerialPort {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
