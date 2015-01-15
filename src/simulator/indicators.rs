use chart::Candle;
use lua;

#[derive(Clone,Copy,Show)]
pub enum IndicatorType {
    SMA,
}

#[derive(Clone)]
pub struct Indicator {
    name: String,
    num_candles: i32,
    indicator_type: IndicatorType,
}

impl Indicator {
    fn new(name: &str, num_candles: i32, it: IndicatorType) -> Indicator {
        Indicator {
            name: name.to_string(),
            num_candles: num_candles,
            indicator_type: it,
        }
    }

    pub fn new_sma(name: &str, num_candles: i32) -> Indicator {
        Indicator::new(name, num_candles, IndicatorType::SMA)
    }

    pub fn update(&mut self, candles: &Vec<Candle>, send_to_lua: bool) {
        match self.indicator_type {
            IndicatorType::SMA => {
                let mut avg = 0.0f32;

                // start at 1 to skip the first incomplete candle
                for i in range(1, self.num_candles as uint) {
                    avg += candles[i].close_bid;
                }

                let new_value = avg / self.num_candles as f32;

                // println!("Updating {} -> {}", self.get_name(), new_value);
                if send_to_lua {
                    lua::register_number(self.name.as_slice(), new_value);
                }
            },
            // _ => panic!("Unknown indicator type: {}", self.indicator_type),
        }
    }
}
