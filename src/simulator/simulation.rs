use std::io::File;

use chart::Chart;
use config;
use tick::Tick;
use trade::Trade;

static mut SIMULATION_ID: i32 = 1;

pub struct Simulation {
    id: i32,
    in_sample: bool,

    jpy_base: bool,

    charts: Vec<Chart>,

    deposit: f32,
    last_equity_high: f32,
    last_equity_low: f32,
    highest_drawdown: f32,
    drawdown_limit: f32,

    open_trades: Vec<Trade>,
    closed_trades: Vec<Trade>,
}

impl Simulation {
    pub fn activate_charts(&mut self) {
        for chart in self.charts.iter_mut() {
            chart.set_active();
        }
    }

    // "balance" is the sum of deposit + closed trades
    pub fn balance(&self) -> f32 {
        let mut balance = self.deposit;

        for trade in self.closed_trades.iter() {
            balance += trade.profit();
        }

        match self.jpy_base {
            true  => balance / 100.0,
            false => balance,
        }
    }

    // returns true when all the attached charts are fully populated with data
    pub fn can_trade(&mut self) -> bool {
        for chart in self.charts.iter_mut() {
            if !chart.has_full_data() {
                return false;
            }
        }

        true
    }

    pub fn close_all_open_trades(&mut self, tick: &Tick) {
        for trade in self.open_trades.iter_mut() {
            trade.close(tick);
        }

        self.migrate_closed_trades();
    }

    pub fn closed_trades_count(&self) -> uint {
        self.closed_trades.len()
    }

    pub fn closed_long_trade_count(&self) -> uint {
        let mut count: uint = 0;

        for trade in self.closed_trades.iter() {
            if trade.is_long() {
                count += 1;
            }
        }

        count
    }

    pub fn closed_short_trade_count(&self) -> uint {
        self.closed_trades_count() - self.closed_long_trade_count()
    }

    pub fn equity(&self) -> f32 {
        // "equity" is the sum of current balance + profit/loss of open trades
        let mut equity = 0.0f32;

        for trade in self.open_trades.iter() {
            equity += trade.profit();
        }

        match self.jpy_base {
            true  => self.balance() + (equity / 100.0),
            false => self.balance() + equity,
        }
    }

    pub fn get_highest_drawdown(&self) -> f32 {
        self.highest_drawdown
    }

    pub fn has_exceeded_max_drawdown(&self) -> bool {
        self.highest_drawdown < self.drawdown_limit
    }

    pub fn has_open_trades(&self) -> bool {
        self.open_trades.len() > 0
    }

    pub fn log_trades(&self, logfile: &mut File) {
        for trade in self.closed_trades.iter() {
            let s = format!("{},{},{},{}", self.id, trade.get_id(), self.in_sample, trade.to_csv());

            logfile.write(s.as_bytes()).ok().unwrap();
        }
    }

    pub fn log_ticks(&self, logfile: &mut File) {
        for trade in self.closed_trades.iter() {
            for tick in trade.ticks.iter() {
                let s = format!("{},{},{}", self.id, trade.get_id(), tick.to_csv());

                logfile.write(s.as_bytes()).ok().unwrap();
            }
        }
    }

    pub fn new(charts: Vec<Chart>, in_sample: bool) -> Simulation {
        // TODO: make this configurable
        let deposit: f32 = 10000.0;

        let jpy_base = config::get().jpy_base;

        if jpy_base {
            println!("NOTE: CSV file has JPY base currency");
        }

        Simulation {
            id: Simulation::next_id(),
            in_sample: in_sample,
            jpy_base: jpy_base,
            deposit: deposit,
            last_equity_high: deposit,
            last_equity_low: deposit,
            closed_trades: vec!(),
            open_trades: vec!(),
            charts: charts,
            drawdown_limit: -10.0, // TODO: make this configurable
            highest_drawdown: 0.0,
        }
    }

    pub fn open_long_trade(&mut self, tick: &Tick) {
        self.record_new_trade(Trade::new_long_trade(tick));
    }

    pub fn open_short_trade(&mut self, tick: &Tick) {
        self.record_new_trade(Trade::new_short_trade(tick));
    }

    pub fn migrate_closed_trades(&mut self) {
        let mut indexes: Vec<uint> = vec!();

        let mut count: uint = 0;

        for trade in self.open_trades.iter() {
            if trade.is_closed() {
                indexes.push(count);
            }

            count += 1;
        }

        indexes.reverse();

        for &index in indexes.iter() {
            let trade = self.open_trades.remove(index);
            self.closed_trades.push(trade);
        }
    }

    pub fn next_id() -> i32 {
        unsafe {
            SIMULATION_ID += 1;
            SIMULATION_ID - 1
        }
    }

    pub fn open_trades_count(&self) -> uint {
        self.open_trades.len()
    }

    pub fn pip_expectancy(&self) -> f32 {
        if 0 != self.open_trades_count() {
            panic!("pip expectancy can only be determined when no orders are open");
        }

        self.profit() / self.closed_trades_count() as f32
    }

    pub fn profit(&self) -> f32 {
        self.balance() - self.deposit
    }

    fn record_new_trade(&mut self, trade: Trade) {
        self.open_trades.push(trade);
    }

    pub fn record_tick_onto_trades(&mut self, tick: &Tick) {
        for trade in self.open_trades.iter_mut() {
            trade.record_tick(tick.clone());
        }
    }

    pub fn update_charts(&mut self, tick: &Tick) {
        for chart in self.charts.iter_mut() {
            chart.process_tick(tick);
        }
    }

    pub fn update_drawdown(&mut self) {
        let equity = self.equity();

        if equity > self.last_equity_high {
            self.last_equity_high = equity;
            self.last_equity_low  = equity;
        } else if equity <= self.last_equity_high {
            self.last_equity_low = equity;
        }

        let drawdown = -(100.0 - ((self.last_equity_low / self.last_equity_high) * 100.0));

        if drawdown < self.highest_drawdown {
            self.highest_drawdown = drawdown;
        }
    }
}
