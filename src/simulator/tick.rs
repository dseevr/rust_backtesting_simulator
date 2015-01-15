extern crate time;

use std::io::File;

use utilities;

#[derive(Clone,Copy)]
pub struct Tick {
    pub time: time::Tm,
    pub bid: f32,
    pub ask: f32,
}

impl Tick {
    pub fn empty_tick() -> Tick {
        Tick::new(time::empty_tm(), 0.0, 0.0)
    }

    pub fn is_sunday(&self) -> bool {
        let year = self.time.tm_year + 1900;
        let month = self.time.tm_mon + 1;
        let day = self.time.tm_mday;

        0 == utilities::day_of_week(year as uint, month as uint, day as uint)
    }

    pub fn new(time: time::Tm, bid: f32, ask: f32) -> Tick {
        Tick { time: time, bid: bid, ask: ask }
    }

    pub fn new_from_line(l: String) -> Tick {
        let mut line = l.clone();

        let length = line.len();
        line.truncate(length - 1);

        let parts: Vec<&str> = line.as_slice().split(',').collect();
        let tm = utilities::tm_from_string(parts[0]);
        let bid = utilities::string_to_float(parts[1]);
        let ask = utilities::string_to_float(parts[2]);

        Tick::new(tm, bid, ask)
    }

    pub fn to_csv(&self) -> String {
        let s = format!(
            "{},{},{}\n",
            self.bid,
            self.ask,
            utilities::tm_to_iso(self.time),
        );

        s.clone()
    }

    pub fn write_csv_header(logfile: &mut File) {
        logfile.write(b"simulation_id,trade_id,bid,ask,time\n").ok().unwrap();
    }
}
