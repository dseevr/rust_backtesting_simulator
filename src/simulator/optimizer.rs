use std::io::File;

use algorithm::Algorithm;
use chart::Chart;
use config;
use range_bound_variable::RangeBoundVariables;
use strategy::Strategy;
use tick::Tick;

pub struct Optimizer {
    strategy: Strategy,
}

impl Optimizer {
    pub fn new(strategy: Strategy) -> Optimizer {
        Optimizer {
            strategy: strategy,
        }
    }

    pub fn variables_for(&self,
                        charts: Vec<Chart>,
                        ticks: &Vec<Tick>,
                        tradelog: &mut File,
                        ticklog: &mut File) -> Option<RangeBoundVariables> {
        let mut best_variables = RangeBoundVariables::new();
        let mut best_score = -999999.0;
        let mut successful_algorithm_count = 0i;

        let max_iterations = config::get().iterations;

        for i in range(1i32, max_iterations + 1) {
            let config_variables = config::get().variables.clone();
            let mut vars = RangeBoundVariables::new_from_string(config_variables);
            vars.randomize();

            let mut algorithm = Algorithm::new_in_sample(self.strategy.clone(), charts.clone());

            println!("-------------------- TEST {} --------------------", i);

            let score: f32 = match algorithm.execute_on(ticks, vars.clone(), tradelog, ticklog) {
                Some(score) => score,
                None        => continue,
            };

            if score > best_score {
                best_score = score;
                best_variables = vars;
            }

            successful_algorithm_count += 1;
        }

        // TODO: Determine a real (probably positive) cutoff for this best_score check.
        //       Even better, make it a config variable.
        if 0 == successful_algorithm_count || best_score < -999.99 {
            None
        } else {
            println!("Best score: {}", best_score);
            Some(best_variables)
        }
    }
}
