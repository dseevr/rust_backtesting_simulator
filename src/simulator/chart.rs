// Lua indexes start at 1 for some reason.
// If the config file specifies 60 periods, they will be FULL candles indexed from 1-60.
// Index 0 will be the current period's incomplete candle.

use lua;
use indicators::Indicator;
use tick::Tick;

#[derive(Clone,Copy)]
pub enum ChartPeriod {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
}

#[derive(Clone,Copy,Show)]
pub enum ChartType {
    Candlestick,
    Renko,
}

// ===== GLOBAL FUNCTIONS ==========================================================================

fn period_from_string(s: &str) -> ChartPeriod {
    match s {
        "M1"  => ChartPeriod::M1,
        "M5"  => ChartPeriod::M5,
        "M15" => ChartPeriod::M15,
        "M30" => ChartPeriod::M30,
        "H1"  => ChartPeriod::H1,
        "H4"  => ChartPeriod::H4,
        _     => panic!("Unknown chart period: {}", s)
    }
}

fn seconds_in_period(period: ChartPeriod) -> i32 {
    match period {
        ChartPeriod::M1  => 60,
        ChartPeriod::M5  => 60 * 5,
        ChartPeriod::M15 => 60 * 15,
        ChartPeriod::M30 => 60 * 30,
        ChartPeriod::H1  => 60 * 60,
        ChartPeriod::H4  => 60 * 60 * 4,
    }
}

// ===== CANDLE ====================================================================================

#[derive(Clone,Copy,Show)]
pub struct Candle {
    open_bid:  f32,
    open_ask:  f32,
    pub close_bid: f32,
    close_ask: f32,

    high_bid: f32,
    high_ask: f32,
    low_bid:  f32,
    low_ask:  f32,

    volume: i32,

    id: i32,
}

// ===== CHART =====================================================================================

#[derive(Clone)]
pub struct Chart {
    candles: Vec<Candle>,
    last_tick: Tick,
    max_candles: i32,
    name: String,
    seconds_per_period: i32,
    indicators: Vec<Indicator>,
    chart_type: ChartType,
    active: bool,
    ticks_processed: i32,
}

impl Chart {
    fn new(name: &str, period: &str, max_candles: i32, ct: ChartType) -> Chart {
        let candles: Vec<Candle> =  Vec::with_capacity(max_candles as uint);
        let period_type = period_from_string(period);

        Chart {
            candles: candles,
            last_tick: Tick::empty_tick(),
            max_candles: max_candles + 1, // see comments at top of file about indexes
            name: name.to_string(),
            seconds_per_period: seconds_in_period(period_type),
            indicators: vec!(),
            chart_type: ct,
            active: false,
            ticks_processed: 0,
        }
    }

    pub fn new_candlestick_chart(name: &str, period: &str, max_candles: i32) -> Chart {
        Chart::new(name, period, max_candles, ChartType::Candlestick)
    }

    pub fn attach_indicator(&mut self, indi: Indicator) {
        self.indicators.push(indi)
    }

    fn create_new_candle_from_tick(&mut self, id: i32, tick: &Tick) {
        if self.candles.len() as i32 >= self.max_candles {
            self.candles.pop();
        }

        let candle = Candle {
            open_bid:  tick.bid,
            open_ask:  tick.ask,
            close_bid: 0.0,
            close_ask: 0.0,

            high_bid: tick.bid,
            high_ask: tick.ask,
            low_bid:  tick.bid,
            low_ask:  tick.ask,

            volume: 1,

            id: id,
        };

        self.candles.insert(0, candle);

        if self.has_full_data() {
            for indicator in self.indicators.iter_mut() {
                indicator.update(&self.candles, self.active);
            }
        }

        self.send_to_lua();
    }

    pub fn has_full_data(&mut self) -> bool {
        self.candles.len() as i32 == self.max_candles
    }

    fn update_latest_candle(&mut self, tick: &Tick) {
        let mut candle = self.candles[0];

        if tick.bid > candle.high_bid {
            candle.high_bid = tick.bid;
        } else if tick.bid < candle.low_bid {
            candle.low_bid = tick.bid;
        }

        if tick.ask > candle.high_ask {
            candle.high_ask = tick.ask;
        } else if tick.ask < candle.low_ask {
            candle.low_ask = tick.ask;
        }

        candle.volume += 1;
    }

    pub fn process_tick(&mut self, tick: &Tick) {
        match self.chart_type {
            ChartType::Candlestick => {
                let num_candles = self.candles.len() as i32;

                let id: i32 = tick.time.to_timespec().sec as i32 / self.seconds_per_period;

                if 0 == num_candles {
                    self.create_new_candle_from_tick(id, tick);
                } else {
                    if id > self.candles[0].id {
                        self.candles[0].close_bid = self.last_tick.bid;
                        self.candles[0].close_ask = self.last_tick.ask;

                        self.create_new_candle_from_tick(id, tick);
                    } else {
                        self.update_latest_candle(tick);
                    }
                }
            },
            _ => panic!("Unknown chart type: {}", self.chart_type),
        };

        self.last_tick = tick.clone();

        self.ticks_processed += 1;
    }

    fn send_to_lua(&self) {
        if !self.active {
            return
        }

        let mut index = 0i32; // see comments at top of file about indexes

        lua::create_table(self.candles.len() as i32);

        for candle in self.candles.iter() {
            lua::push_table_integer(index);
            lua::create_table(5);

            lua::push_table_string("open_bid");
            lua::push_table_number(candle.open_bid);
            lua::set_table(-3);

            lua::push_table_string("open_ask");
            lua::push_table_number(candle.open_ask);
            lua::set_table(-3);

            lua::push_table_string("close_bid");
            lua::push_table_number(candle.close_bid);
            lua::set_table(-3);

            lua::push_table_string("close_ask");
            lua::push_table_number(candle.close_ask);
            lua::set_table(-3);

            lua::push_table_string("volume");
            lua::push_table_integer(candle.volume);
            lua::set_table(-3);

            lua::set_table(-3);
            index += 1;
        }

        lua::finalize_table(self.name.as_slice());
    }

    pub fn set_active(&mut self) {
        self.active = true;
    }

    pub fn get_ticks_processed(&self) -> i32 {
        self.ticks_processed
    }
}
