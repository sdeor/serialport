use winapi::{
    shared::minwindef::DWORD,
    um::{commapi, winbase, winnt::HANDLE},
};

use crate::builder::SerialPortBuilder;
use crate::config::{DataBits, FlowControl, Parity, StopBits};

use super::winapi_result;

#[must_use]
pub(super) struct WindowsDCB {
    pub(super) inner: winbase::DCB,
}

impl WindowsDCB {
    pub fn get(handle: HANDLE) -> std::io::Result<Self> {
        let mut dcb: winbase::DCB = unsafe { std::mem::zeroed() };
        dcb.DCBlength = std::mem::size_of::<winbase::DCB>() as DWORD;

        winapi_result(unsafe { commapi::GetCommState(handle, &mut dcb) })?;

        Ok(Self { inner: dcb })
    }

    pub fn update(&mut self, builder: &SerialPortBuilder) -> &mut Self {
        self.baud_rate(builder.baud_rate)
            .data_bits(builder.data_bits)
            .stop_bits(builder.stop_bits)
            .parity(builder.parity)
            .flow_control(builder.flow_control)
    }

    pub fn baud_rate(&mut self, baud_rate: u32) -> &mut Self {
        self.inner.BaudRate = baud_rate;
        self
    }

    pub fn data_bits(&mut self, data_bits: DataBits) -> &mut Self {
        self.inner.ByteSize = match data_bits {
            DataBits::Five => 5,
            DataBits::Six => 6,
            DataBits::Seven => 7,
            DataBits::Eight => 8,
        };
        self
    }

    pub fn stop_bits(&mut self, stop_bits: StopBits) -> &mut Self {
        self.inner.StopBits = match stop_bits {
            StopBits::One => winbase::ONESTOPBIT,
            StopBits::OnePointFive => winbase::ONE5STOPBITS,
            StopBits::Two => winbase::TWOSTOPBITS,
        };
        self
    }

    pub fn parity(&mut self, parity: Parity) -> &mut Self {
        self.inner.Parity = match parity {
            Parity::None => winbase::NOPARITY,
            Parity::Odd => winbase::ODDPARITY,
            Parity::Even => winbase::EVENPARITY,
            Parity::Mark => winbase::MARKPARITY,
            Parity::Space => winbase::SPACEPARITY,
        };
        self
    }

    pub fn flow_control(&mut self, flow_control: FlowControl) -> &mut Self {
        match flow_control {
            FlowControl::None => {
                self.inner.set_fOutxCtsFlow(0);
                self.inner.set_fRtsControl(winbase::RTS_CONTROL_DISABLE);
                self.inner.set_fOutX(0);
                self.inner.set_fInX(0);
            }
            FlowControl::Software => {
                self.inner.set_fOutxCtsFlow(0);
                self.inner.set_fRtsControl(winbase::RTS_CONTROL_DISABLE);
                self.inner.set_fOutX(1);
                self.inner.set_fInX(1);
            }
            FlowControl::Hardware => {
                self.inner.set_fOutxCtsFlow(1);
                self.inner.set_fRtsControl(winbase::RTS_CONTROL_HANDSHAKE);
                self.inner.set_fOutX(0);
                self.inner.set_fInX(0);
            }
        }
        self
    }

    pub fn set(&mut self, handle: HANDLE) -> std::io::Result<()> {
        winapi_result(unsafe { commapi::SetCommState(handle, &mut self.inner) })
    }
}
