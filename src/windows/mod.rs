use windows_sys::Win32::{
    Foundation::{GetLastError, SYSTEMTIME},
    System::SystemInformation::SetSystemTime,
};

use crate::Time;

impl From<Time> for SYSTEMTIME {
    fn from(value: Time) -> Self {
        SYSTEMTIME {
            wYear: value.year,
            wMonth: value.month as u16,
            wDayOfWeek: value.day_of_week as u16,
            wDay: value.day as u16,
            wHour: value.hour as u16,
            wMinute: value.minute as u16,
            wSecond: value.second as u16,
            wMilliseconds: 0,
        }
    }
}

pub fn set_system_time(time: Time) -> Result<(), String> {
    unsafe {
        match SetSystemTime(&time.into()) {
            0 => Err(format!("Error in SetSystemTime: {}", GetLastError())),
            _ => Ok(()),
        }
    }
}
