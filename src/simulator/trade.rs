extern crate time;

// use std::fmt;
use std::io::File;

use tick::Tick;
use utilities;

#[derive(Copy)]
pub enum TradeDirection {
    LONG,
    SHORT,
}

// TODO: Shouldn't this work? Gets this error: rustc --explain E0001
// impl fmt::Show for TradeDirection {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             LONG  => write!(f, "long"),
//             SHORT => write!(f, "short"),
//         }
//     }
// }

static mut TRADE_ID: i32 = 1;

pub struct Trade {
    id: i32,

    open: bool,

    pub direction: TradeDirection,

    pub opened_at: time::Tm,
    pub closed_at: time::Tm,

    pub open_price:  f32, // price filled at
    pub close_price: f32, // price filled at
    // desired_open_price: f32,
    // desired_close_price: f32,

    // allow_slippage: f32, // TODO: implement this

    // stopLoss   []stops.StopLoss
    // takeProfit []stops.TakeProfit

    // stop_loss_hit: bool,
    // take_profit_hit: bool,

    // for calculating spreads and actual slippage
    pub open_bid: f32,
    pub open_ask: f32,
    pub close_bid: f32,
    pub close_ask: f32,

    // extra fields for debugging / analytics
    // EquityAtOpen   float64
    // EquityAtClose  float64
    // BalanceAtOpen  float64
    // BalanceAtClose float64

    // LowestBid      float64
    // LowestAsk      float64
    // HighestBid     float64
    // HighestAsk     float64

    // OrdersOpenAtOpen  int64
    // OrdersOpenAtClose int64

    // DrawdownAtOpen  float64
    // DrawdownAtClose float64

    // ClosestPercentageToTakeProfit float64
    // ClosestPercentageToStopLoss   float64

    pub ticks: Vec<Tick>,
}

impl Trade {
    pub fn close(&mut self, tick: &Tick) {
        if self.is_closed() {
            panic!("can't close a closed trade")
        }
        let close_price = match self.is_long() {
            true  => tick.bid,
            false => tick.ask,
        };

        self.closed_at = tick.time;
        self.close_price = close_price;
        self.close_bid = tick.bid;
        self.close_ask = tick.ask;

        self.open = false;
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }

    pub fn is_long(&self) -> bool {
        match self.direction {
            TradeDirection::LONG  => true,
            TradeDirection::SHORT => false,
        }
    }

    pub fn is_short(&self) -> bool {
        !self.is_long()
    }

    pub fn new_long_trade(tick: &Tick) -> Trade {
        Trade::new(tick, TradeDirection::LONG)
    }

    pub fn new_short_trade(tick: &Tick) -> Trade {
        Trade::new(tick, TradeDirection::SHORT)
    }

    fn new(tick: &Tick, direction: TradeDirection) -> Trade {
        let open_price = match direction {
            TradeDirection::LONG  => tick.ask,
            TradeDirection::SHORT => tick.bid,
        };

        let ticks: Vec<Tick> = vec!();

        let tickaroo = tick.clone();

        let mut t = Trade {
            id: Trade::next_id(),
            open: true,

            direction: direction,

            opened_at: tickaroo.time,
            closed_at: time::empty_tm(),

            open_price:  open_price,
            close_price: 0.0,
            open_bid: tickaroo.bid,
            open_ask: tickaroo.ask,
            close_bid: 0.0,
            close_ask: 0.0,

            ticks: ticks,
        };

        t.record_tick(tick.clone());

        t
    }

    pub fn next_id() -> i32 {
        unsafe {
            TRADE_ID += 1;
            TRADE_ID - 1
        }
    }

    pub fn profit(&self) -> f32 {
        if self.is_open() {
            let last_tick = match self.ticks.last() {
                Some(val) => val,
                None      => panic!("an open trade has no ticks?"),
            };

            match self.is_long() {
                true  => utilities::pip_profit(self.open_price, last_tick.bid),
                false => utilities::pip_profit(last_tick.ask, self.open_price),
            }
        } else {
            match self.is_long() {
                true  => utilities::pip_profit(self.open_price, self.close_price),
                false => utilities::pip_profit(self.close_price, self.open_price),
            }
        }
    }

    pub fn record_tick(&mut self, tick: Tick) {
        self.ticks.push(tick);
    }

    pub fn write_csv_header(logfile: &mut File) {
        logfile.write(b"simulation_id,trade_id,in_sample,long,opened_at,closed_at,").ok().unwrap();
        logfile.write(b"open_price,close_price,open_spread,close_spread,profit\n").ok().unwrap();
    }

    pub fn to_csv(&self) -> String {
        let long = match self.direction {
            TradeDirection::LONG  => true,
            TradeDirection::SHORT => false,
        };

        let s = format!(
            "{},{},{},{},{},{},{},{:.1}\n",
            long,
            utilities::tm_to_iso(self.opened_at),
            utilities::tm_to_iso(self.closed_at),
            self.open_price,
            self.close_price,
            self.open_ask  - self.open_bid,
            self.close_ask - self.close_bid,
            self.profit(),
        );

        s.clone()
    }
}
