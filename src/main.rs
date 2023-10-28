use ureq;
use native_tls;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::set_system_time;

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

fn parse_http_response_date(date: &str) -> Result<Time, String> {
    // Sat, 09 Oct 2010 14:28:02 GMT
    fn error(date: &str, step: &str) -> Result<Time, String> {
        Err(format!("Bad field <{step}> in \"{date}\"",))
    }
    let mut parts = date.split(|c| c == ' ' || c == ':');
    let day_of_week = match parts.next() {
        Some("Mon,") => 1,
        Some("Tue,") => 2,
        Some("Wed,") => 3,
        Some("Thu,") => 4,
        Some("Fri,") => 5,
        Some("Sat,") => 6,
        Some("Sun,") => 0,
        _ => return error(date, "day_of_week"),
    };
    let day = match parts.next().map(|d| d.parse::<u8>()) {
        Some(Ok(d)) => d,
        _ => return error(date, "day"),
    };
    let month = match parts.next() {
        Some("Jan") => 1,
        Some("Feb") => 2,
        Some("Mar") => 3,
        Some("Apr") => 4,
        Some("May") => 5,
        Some("Jun") => 6,
        Some("Jul") => 7,
        Some("Aug") => 8,
        Some("Sep") => 9,
        Some("Oct") => 10,
        Some("Nov") => 11,
        Some("Dec") => 12,
        _ => return error(date, "month"),
    };
    let year = match parts.next().map(|d| d.parse::<u16>()) {
        Some(Ok(d)) => d,
        _ => return error(date, "year"),
    };
    let hour = match parts.next().map(|d| d.parse::<u8>()) {
        Some(Ok(d)) => d,
        _ => return error(date, "hour"),
    };
    let minute = match parts.next().map(|d| d.parse::<u8>()) {
        Some(Ok(d)) => d,
        _ => return error(date, "minute"),
    };
    let second = match parts.next().map(|d| d.parse::<u8>()) {
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

pub fn get_time_from_response(response: &ureq::Response) -> Result<Time, String> {
    Ok(parse_http_response_date(
        response.header("Date").ok_or("No Date field in response")?,
    )?)
}

pub fn get_time_from_url(url: &str) -> Result<Time, String> {
    let agent = ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(
            native_tls::TlsConnector::new().map_err(|e| format!("native_tls error: {e}"))?,
        ))
        .build();
    match agent.get(url).call() {
        Ok(response) => {
            Ok(get_time_from_response(&response)
                .map_err(|e| format!("Response parse error: {e}"))?)
        }
        Err(ureq::Error::Status(_code, response)) => {
            Ok(get_time_from_response(&response)
                .map_err(|e| format!("Response parse error: {e}"))?)
        }
        Err(error) => Err(format!("Request error: {error}")),
    }
}

pub fn run(url: &str) -> Result<(), String> {
    let time = get_time_from_url(url)?;
    //println!("{}:{}", time.hour, time.minute); Ok(())
    set_system_time(time)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 2 {
        if let Err(e) = run(&args[1]) {
            eprintln!("{e}");
        }
    } else {
        println!("settime <URL>");
    }
}
