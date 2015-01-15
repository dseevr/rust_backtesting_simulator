use lua;
use range_bound_variable::RangeBoundVariables;
use simulation::Simulation;
use tick::Tick;

#[derive(Clone)]
pub struct Strategy {
    path: String,
}

impl Strategy {
    pub fn new(path: &str) -> Strategy {
        Strategy {
            path: path.to_string(),
        }
    }

    pub fn on_tick(&self, sim: &mut Simulation, tick: &Tick) {
        lua::register_number("current_bid", tick.bid);
        lua::register_number("current_ask", tick.ask);
        lua::register_number("current_spread", tick.ask - tick.bid);

        lua::register_boolean("has_open_trades", sim.has_open_trades());

        match lua::on_tick() {
            lua::TradeDecision::LONG  => sim.open_long_trade(tick),
            lua::TradeDecision::SHORT => sim.open_short_trade(tick),
            lua::TradeDecision::CLOSE => sim.close_all_open_trades(tick),
            lua::TradeDecision::NOOP  => {}
        }
    }

    pub fn setup(&self, vars: RangeBoundVariables) {
        lua::setup(self.path.as_slice());

        vars.register_in_lua();

        println!("Registered variables:");
        vars.print();
    }

    pub fn teardown(&self) {
        lua::teardown();
    }
}
