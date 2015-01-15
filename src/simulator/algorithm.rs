use std::io::File;

use chart::Chart;
use range_bound_variable::RangeBoundVariables;
use simulation::Simulation;
use strategy::Strategy;
use tick::Tick;

pub struct Algorithm {
    simulation: Simulation,
    strategy: Strategy,
    in_sample: bool,
}

impl Algorithm {
    pub fn execute_on(&mut self,
                      ticks: &Vec<Tick>,
                      vars: RangeBoundVariables,
                      tradefile: &mut File,
                      tickfile: &mut File) -> Option<f32> {
        let mut tick_count = 0i32;
        let mut exceeded_drawdown_limit = false;

        let ref mut sim = self.simulation;

        self.strategy.setup(vars);
        sim.activate_charts();

        let mut last_tick = &Tick::empty_tick();

        for tick in ticks.iter() {
            tick_count += 1;

            sim.record_tick_onto_trades(tick);
            sim.update_charts(tick);

            sim.update_drawdown();
            if sim.has_exceeded_max_drawdown() {
                exceeded_drawdown_limit = true;
                break;
            }
            // TODO: self.simulation.process_stops();
            // TODO: in pre-tick SL/TP checks, make sure FIFO is not violated

            if sim.can_trade() {
                self.strategy.on_tick(sim, tick);
            }

            last_tick = tick;
        }

        // TODO: BUG. This will record the last tick again onto the trades.
        sim.close_all_open_trades(last_tick);
        self.strategy.teardown();

        sim.log_trades(tradefile);
        sim.log_ticks(tickfile);

        if exceeded_drawdown_limit {
                println!("Exceeded max drawdown, aborting simulation");
                None
        } else {
            println!(
                "Final score: {:.1} - Profit: {:.1} - Total trades: {}/{}",
                sim.pip_expectancy(),
                sim.profit(),
                sim.closed_long_trade_count(),
                sim.closed_short_trade_count(),
            );

            println!("Ticks processed: {} - Max DD: {:.2}%",
                tick_count,
                sim.get_highest_drawdown(),
            );

            Some(sim.pip_expectancy())
        }
    }

    pub fn new_in_sample(strategy: Strategy, charts: Vec<Chart>) -> Algorithm {
        Algorithm::new(strategy, charts, true)
    }

    pub fn new_out_of_sample(strategy: Strategy, charts: Vec<Chart>) -> Algorithm {
        Algorithm::new(strategy, charts, false)
    }

    pub fn new(strategy: Strategy, charts: Vec<Chart>, in_sample: bool) -> Algorithm {
        Algorithm {
            simulation: Simulation::new(charts, in_sample),
            strategy: strategy,
            in_sample: in_sample,
        }
    }
}
