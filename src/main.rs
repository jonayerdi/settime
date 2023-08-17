use std::{
    env,
    io::{Read, Write},
    net::TcpStream, fmt::Display,
};

#[cfg(target_os="windows")]
mod windows;
#[cfg(target_os="windows")]
use windows::set_system_time;

const HTTP_RESPONSE_DATE_PREFIX: &str = "Date: ";

struct ByteStr<'a>(&'a [u8]);

impl Display for ByteStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &c in self.0 {
            write!(f, "{}", c as char)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Time {
    year: u16,
    month: u8,
    day_of_week: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

fn parse_u8(num: &[u8]) -> Result<u8,()> {
    let mut result = 0;
    for &digit in num {
        let digit = digit.wrapping_sub(b'0');
        if digit > 9 {
            return Err(());
        }
        result = result * 10 + (digit as u8);
    }
    Ok(result)
}

fn parse_u16(num: &[u8]) -> Result<u16,()> {
    let mut result = 0;
    for &digit in num {
        let digit = digit.wrapping_sub(b'0');
        if digit > 9 {
            return Err(());
        }
        result = result * 10 + (digit as u16);
    }
    Ok(result)
}

fn parse_http_response_date(date: &[u8]) -> Result<Time, String> {
    // Sat, 09 Oct 2010 14:28:02 GMT
    fn error(date: &[u8], step: &str) -> Result<Time,String> {
        Err(format!("Error parsing date from HTTP response ({}): {}", step, ByteStr(date)))
    }
    let mut parts = date.split(|&c| c == b' ' || c == b':');
    let day_of_week = match parts.next() {
        Some(b"Mon,") => 1,
        Some(b"Tue,") => 2,
        Some(b"Wed,") => 3,
        Some(b"Thu,") => 4,
        Some(b"Fri,") => 5,
        Some(b"Sat,") => 6,
        Some(b"Sun,") => 0,
        _ => return error(date, "day_of_week"),
    };
    let day = match parts.next().map(|d| parse_u8(d)) {
        Some(Ok(d)) => d,
        _ => return error(date, "day"),
    };
    let month = match parts.next() {
        Some(b"Jan") => 1,
        Some(b"Feb") => 2,
        Some(b"Mar") => 3,
        Some(b"Apr") => 4,
        Some(b"May") => 5,
        Some(b"Jun") => 6,
        Some(b"Jul") => 7,
        Some(b"Aug") => 8,
        Some(b"Sep") => 9,
        Some(b"Oct") => 10,
        Some(b"Nov") => 11,
        Some(b"Dec") => 12,
        _ => return error(date, "month"),
    };
    let year = match parts.next().map(|d| parse_u16(d)) {
        Some(Ok(d)) => d,
        _ => return error(date, "year"),
    };
    let hour = match parts.next().map(|d| parse_u8(d)) {
        Some(Ok(d)) => d,
        _ => return error(date, "hour"),
    };
    let minute = match parts.next().map(|d| parse_u8(d)) {
        Some(Ok(d)) => d,
        _ => return error(date, "minute"),
    };
    let second = match parts.next().map(|d| parse_u8(d)) {
        Some(Ok(d)) => d,
        _ => return error(date, "second"),
    };
    Ok(Time {
        year,
        month,
        day_of_week,
        day,
        hour,
        minute,
        second,
    })
}

fn get_http_response_line_date(line: &[u8]) -> Option<Result<Time, String>> {
    if line.starts_with(HTTP_RESPONSE_DATE_PREFIX.as_bytes()) {
        Some(parse_http_response_date(&line[HTTP_RESPONSE_DATE_PREFIX.as_bytes().len()..]))
    } else {
        None
    }
}

fn get_http_response_date(response: &mut Vec<u8>) -> Option<Result<Time, String>> {
    let mut line_start = 0;
    for index in 0..response.len() {
        if response[index] == '\n' as u8 {
            if let Some(time) = get_http_response_line_date(&response[line_start..index]) {
                return Some(time);
            } else {
                line_start = index + 1;
            }
        }
    }
    // TODO: if line_start != 0 { /* Remove already processed lines */ }
    None
}

pub fn get_time_http(url: &str) -> Result<Time, String> {
    let mut stream =
        TcpStream::connect(url).map_err(|e| format!("Error connecting to \"{url}\": {e}"))?;
    stream
        .write_all(format!("GET / HTTP/1.1\n\n").as_bytes())
        .map_err(|e| format!("Error writing http request to \"{url}\": {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("Error flushing TCP connection with \"{url}\": {e}"))?;
    let mut lines = Vec::with_capacity(512);
    let mut buffer = [0u8; 512];
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|e| format!("Error reading from TCP connection with \"{url}\": {e}"))?;
        lines.extend(&buffer[..read]);
        if let Some(date) = get_http_response_date(&mut lines) {
            return date;
        }
    }
}

pub fn run(timespec: &str) -> Result<(), String> {
    let time = get_time_http(timespec)?;
    //println!("{}:{}", time.hour, time.minute);
    set_system_time(time)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 2 {
        if let Err(e) = run(&args[1]) {
            eprintln!("{e}");
        }
    } else {
        println!("settime <URL>:<PORT>");
    }
}
