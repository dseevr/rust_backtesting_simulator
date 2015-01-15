extern crate time;

use std::io::{BufferedReader,File,SeekStyle};

pub fn buf_reader_from_file(file_path: &str, offset: uint) -> BufferedReader<File> {
    let path = Path::new(file_path);
    let mut fd = match File::open(&path).ok() {
        Some(val) => val,
        None      => panic!("can't open file at path: {}", file_path)
    };

    if offset > 0 {
        fd.seek(offset as i64, SeekStyle::SeekSet).ok().unwrap();
    }

    println!("New reader of {} at offet {}", file_path, offset);

    BufferedReader::new(fd)
}

pub fn day_of_week(y: uint, m: uint, d: uint) -> uint {
    let t: Vec<uint> = vec!(0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4);

    let mut yy = y;

    if m < 3 {
        yy -= 1;
    }

    return (yy + yy/4 - yy/100 + yy/400 + t[m-1] + d) % 7;
}

pub fn split_csv_string(s: &str, delim: char) -> Vec<&str> {
    s.as_slice().split(delim).collect()
}

pub fn csv_is_jpy_base(csv_path: &str) -> bool {
    let path = Path::new(csv_path.as_slice());
    let fd = match File::open(&path).ok() {
        Some(val) => val,
        None      => panic!("can't open file at path: {}", csv_path)
    };

    let mut file = BufferedReader::new(fd);

    let line = file.read_line().ok().unwrap();

    let parts: Vec<&str> = line.as_slice().split(',').collect();
    let number_parts: Vec<&str> = parts.last().unwrap().as_slice().split('.').collect(); 

    // subtract 1 because of the newline on the end
    let places = number_parts.last().unwrap().len() - 1;

    match places {
        3 => true,
        5 => false,
        n => panic!("expected 3 or 5 decimals points, got {} (\"{}\")", n, places),
    }
}

pub fn string_to_int(s: &str) -> i32 {
    s.parse::<i32>().unwrap()
}

pub fn string_to_float(s: &str) -> f32 {
    s.parse::<f32>().unwrap()
}

pub fn tm_to_iso(t: time::Tm) -> String {
    t.strftime("%Y-%m-%d %H:%M:%S").ok().unwrap().to_string()
}

// Converts something like "01/02/2014 04:59:15" into a UNIX timestamp
pub fn timespec_from_string(tick_time: &str) -> time::Timespec {
    tm_from_string(tick_time).to_timespec()
}

// Converts something like "01/02/2014 04:59:15" into a time::Tm
pub fn tm_from_string(tick_time: &str) -> time::Tm {
    match time::strptime(tick_time, "%m/%d/%Y %H:%M:%S") {
        Err(why) => panic!("{}: {}", why, tick_time),
        Ok(tm) => tm,
    }
}

pub fn pip_profit(open_price: f32, close_price: f32) -> f32 {
    // TODO: support JPY base currencies here, multiply by 100~
    (close_price * 10000.0) - (open_price * 10000.0)
}

pub fn config_time_to_days(s: &str) -> i32 {
    let parts: Vec<&str> = s.split(' ').collect();

    if parts.len() != 2 {
        panic!("sample format must be something like \"2 weeks\"");
    }

    let num = string_to_int(parts[0]);
    let period = parts[1];

    if num < 1 {
        panic!("number of periods must be > 0");
    }

    match period {
        "week"  | "weeks"  => num * 7,
        _                  => panic!("Unknown period specified: \"{}\"", period),
    }
}
