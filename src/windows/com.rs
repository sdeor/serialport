use std::io;

use winapi::shared::minwindef::{BOOL, DWORD, LPVOID};
use winapi::um::{
    commapi, fileapi,
    handleapi::{self, INVALID_HANDLE_VALUE},
    processthreadsapi::GetCurrentProcess,
    winbase,
    winnt::{
        DUPLICATE_SAME_ACCESS, FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE, HANDLE, MAXDWORD,
    },
};

use crate::{
    SerialPort, SerialPortBuilder,
    communication::Communication,
    config::{ClearBuffer, DataBits, FlowControl, Parity, StopBits},
    private,
    windows::dcb,
};

pub(super) fn winapi_result(result: BOOL) -> io::Result<()> {
    match result {
        0 => Err(std::io::Error::last_os_error()),
        _ => Ok(()),
    }
}

pub struct ComPort {
    is_open: bool,
    handle: HANDLE,
    builder: SerialPortBuilder,
}

impl ComPort {
    pub fn new(builder: SerialPortBuilder) -> io::Result<Self> {
        let mut serialport = Self {
            is_open: false,
            handle: INVALID_HANDLE_VALUE,
            builder,
        };

        if !serialport.builder.path.is_empty() {
            serialport.open()?;
        }

        Ok(serialport)
    }

    pub fn try_clone_native(&self) -> io::Result<Self> {
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

        Ok(Self {
            is_open: self.is_open,
            handle,
            builder: self.builder.clone(),
        })
    }

    fn reconfigure(&mut self) -> io::Result<()> {
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
}

impl Communication for ComPort {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn open(&mut self) -> io::Result<()> {
        if self.builder.path.is_empty() {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }

        if self.is_open {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }

        let path = if self.builder.path.starts_with(r"\\.\") {
            self.builder.path.clone()
        } else {
            format!(r"\\.\{}", self.builder.path)
        };

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

    fn close(&mut self) -> io::Result<()> {
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
}

impl SerialPort for ComPort {
    fn try_clone(&self) -> io::Result<Box<dyn SerialPort>> {
        self.try_clone_native()
            .map(|port| Box::new(port) as Box<dyn SerialPort>)
    }

    fn path(&self) -> Option<String> {
        Some(self.builder.path.clone())
    }

    fn baud_rate(&self) -> io::Result<u32> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        Ok(dcb.inner.BaudRate)
    }

    fn data_bits(&self) -> io::Result<DataBits> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        match dcb.inner.ByteSize {
            5 => Ok(DataBits::Five),
            6 => Ok(DataBits::Six),
            7 => Ok(DataBits::Seven),
            8 => Ok(DataBits::Eight),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    fn flow_control(&self) -> io::Result<FlowControl> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        if dcb.inner.fOutxCtsFlow() != 0 || dcb.inner.fRtsControl() != winbase::RTS_CONTROL_DISABLE
        {
            Ok(FlowControl::Hardware)
        } else if dcb.inner.fOutX() != 0 || dcb.inner.fInX() != 0 {
            Ok(FlowControl::Software)
        } else {
            Ok(FlowControl::None)
        }
    }

    fn parity(&self) -> io::Result<Parity> {
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

    fn stop_bits(&self) -> io::Result<StopBits> {
        let dcb = dcb::WindowsDCB::get(self.handle)?;
        match dcb.inner.StopBits {
            winbase::ONESTOPBIT => Ok(StopBits::One),
            winbase::ONE5STOPBITS => Ok(StopBits::OnePointFive),
            winbase::TWOSTOPBITS => Ok(StopBits::Two),
            _ => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }

    fn timeout(&self) -> std::time::Duration {
        self.builder.timeout
    }

    fn bytes_to_read(&self) -> io::Result<u32> {
        let mut errors: DWORD = 0;
        let mut comstat = winbase::COMSTAT {
            cbInQue: 0,
            cbOutQue: 0,
            BitFields: 0,
        };

        winapi_result(unsafe { commapi::ClearCommError(self.handle, &mut errors, &mut comstat) })?;

        Ok(comstat.cbInQue)
    }

    fn bytes_to_write(&self) -> io::Result<u32> {
        let mut errors: DWORD = 0;
        let mut comstat = winbase::COMSTAT {
            cbInQue: 0,
            cbOutQue: 0,
            BitFields: 0,
        };

        winapi_result(unsafe { commapi::ClearCommError(self.handle, &mut errors, &mut comstat) })?;

        Ok(comstat.cbOutQue)
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> io::Result<()> {
        self.builder.baud_rate = baud_rate;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn set_data_bits(&mut self, data_bits: DataBits) -> io::Result<()> {
        self.builder.data_bits = data_bits;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn set_flow_control(&mut self, flow_control: FlowControl) -> io::Result<()> {
        self.builder.flow_control = flow_control;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn set_parity(&mut self, parity: Parity) -> io::Result<()> {
        self.builder.parity = parity;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn set_stop_bits(&mut self, stop_bits: StopBits) -> io::Result<()> {
        self.builder.stop_bits = stop_bits;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn set_timeout(&mut self, timeout: std::time::Duration) -> io::Result<()> {
        self.builder.timeout = timeout;

        if self.is_open {
            self.reconfigure()?;
        }

        Ok(())
    }

    fn clear(&self, buffer_to_clear: ClearBuffer) -> io::Result<()> {
        let buffer_flags = match buffer_to_clear {
            ClearBuffer::Input => winbase::PURGE_RXABORT | winbase::PURGE_RXCLEAR,
            ClearBuffer::Output => winbase::PURGE_TXABORT | winbase::PURGE_TXCLEAR,
            ClearBuffer::All => {
                winbase::PURGE_RXABORT
                    | winbase::PURGE_RXCLEAR
                    | winbase::PURGE_TXABORT
                    | winbase::PURGE_TXCLEAR
            }
        };

        winapi_result(unsafe { commapi::PurgeComm(self.handle, buffer_flags) })
    }
}

impl private::Private for ComPort {
    fn set_raw_path<'a>(&mut self, path: std::borrow::Cow<'a, str>) -> io::Result<()> {
        self.builder.path = path.into_owned();
        Ok(())
    }
}

unsafe impl Send for ComPort {}

impl std::io::Read for ComPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
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

impl std::io::Write for ComPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
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

    fn flush(&mut self) -> io::Result<()> {
        if !self.is_open {
            return Err(std::io::ErrorKind::NotConnected.into());
        }

        winapi_result(unsafe { fileapi::FlushFileBuffers(self.handle) })?;

        Ok(())
    }
}

impl Drop for ComPort {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
