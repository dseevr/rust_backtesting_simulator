extern crate simulator;
extern crate time;

use std::io::{Command,File,fs};
use std::os;

use simulator::Algorithm;
use simulator::Chart;
use simulator::config;
use simulator::config::ConfigurationFile;
use simulator::Optimizer;
use simulator::parsers;
use simulator::Strategy;
use simulator::Tick;
use simulator::Trade;
use simulator::utilities;

// ===== PROGRAM ENTRYPOINT ========================================================================

// TODO: figure out wtf is going on with these warnings... compiler error?
#[allow(unused_assignments)]
fn main() {
    println!("==================== SETUP ====================");

    let args = os::args();

    match args.len() {
        1 => panic!("You must provide a Lua file, e.g., {} foo.lua", args[0]),
        2 => (),
        _ => println!("WARNING: additional command line parameters after the Lua file are ignored."),
    }

    let lua_path = args[1].as_slice();

    ConfigurationFile::load_from_file(lua_path);

    let csv_path           = config::get().csv_path.as_slice();
    let in_sample_days     = utilities::config_time_to_days(config::get().in_sample.as_slice());
    let out_of_sample_days = utilities::config_time_to_days(config::get().out_of_sample.as_slice());

    let post_run_script    = config::get().post_run_script.as_slice();

    if utilities::csv_is_jpy_base(csv_path.as_slice()) {
        config::get().set_jpy_base();
    }

    let mut scores: Vec<f32> = vec!();
    let max_steps = config::get().steps;

    let mut ticks_processed_by_charts = 0i32;

    println!("Simulating a maximum of {} steps", max_steps);

    let mut file = utilities::buf_reader_from_file(csv_path, 0);
    let mut bytes_read: uint = 0;

    // ----- FILL CHARTS ---------------------------------------------------------------------------

    println!("==================== FILLING CHARTS ====================");

    let chart_string = config::get().charts.clone();
    let mut charts: Vec<Chart> = parsers::parse_charts_from_string(chart_string);

    for mut line in file.lines().filter_map( |l| l.ok() ) {
        bytes_read += line.len();

        let tick = Tick::new_from_line(line);

        for chart in charts.iter_mut() {
            chart.process_tick(&tick);
            ticks_processed_by_charts += 1;
        }

        let mut all_charts_have_data = true;

        for chart in charts.iter_mut() {
            if !chart.has_full_data() {
                all_charts_have_data = false;
                break;
            }
        }

        if all_charts_have_data {
            break;
        }
    }

    // ----- FIND NEXT SUNDAY ----------------------------------------------------------------------

    println!("==================== ADVANCING TO NEXT SUNDAY ====================");

    let mut start_day = 0i32;
    let mut day_changed = false;
    let mut first_day_check_tick = true;
    let mut found_sunday = false;

    for mut line in file.lines().filter_map( |l| l.ok() ) {
        bytes_read += line.len();

        let tick = Tick::new_from_line(line);

        for chart in charts.iter_mut() {
            chart.process_tick(&tick);
            ticks_processed_by_charts += 1;
        }

        if first_day_check_tick {
            first_day_check_tick = false;

            start_day = tick.time.tm_mday;
            // println!("Start day: {}", utilities::tm_to_iso(tick.time));
            continue;
        }

        if !day_changed && tick.time.tm_mday != start_day {
            day_changed = true;
        }

        if day_changed && tick.is_sunday() {
            // println!("Advanced to {}", utilities::tm_to_iso(tick.time));
            found_sunday = true;
            break;
        }
    }

    if !found_sunday {
        panic!("Advanced to end of file but did not find a Sunday");
    }

    // println!("Read {} bytes so far", bytes_read);

    // ----- SET UP LOGGING ------------------------------------------------------------------------

    let trade_log_path = &Path::new("output/trades.csv");
    let ticks_log_path = &Path::new("output/ticks.csv");

    let _ = fs::unlink(trade_log_path);
    let _ = fs::unlink(ticks_log_path);

    let mut trades_log = File::create(trade_log_path).ok().unwrap();
    let mut ticks_log  = File::create(ticks_log_path).ok().unwrap();

    Trade::write_csv_header(&mut trades_log);
    Tick::write_csv_header(&mut ticks_log);

    // ----- MAIN LOOP -----------------------------------------------------------------------------

    let mut failed_to_optimize_algorithm = false;
    let mut failed_to_execute = false;

    let mut ticks: Vec<Tick> = vec!();

    let mut bytes_read_at_in_sample: uint = 0;
    let mut bytes_read_at_out_of_sample: uint = 0;

    let mut pristine_charts = charts.clone();

    for i in range(1i32, max_steps + 1) {
        let strategy = Strategy::new(lua_path);
        let optimizer = Optimizer::new(strategy.clone());

        let mut first_tick = true;

        let mut num_days = 0i32;
        let mut current_day_number = 0i32;
        let mut target_num_days = 0i32;

        if i > 1 {
            println!("==================== WALKING FORWARD TO NEXT IN SAMPLE ====================");
            bytes_read = bytes_read_at_in_sample;
            // println!("Rewinding to stored cursor at beginning of in sample period");
            file = utilities::buf_reader_from_file(csv_path, bytes_read_at_in_sample);

            // println!("Next tick: {}", utilities::tm_to_iso(Tick::new_from_line(file.read_line().ok().unwrap()).time));

            first_tick = true;
            num_days = 0;
            target_num_days = out_of_sample_days;

            for mut line in file.lines().filter_map( |l| l.ok() ) {
                bytes_read += line.len();

                let tick = Tick::new_from_line(line);

                for chart in pristine_charts.iter_mut() {
                    chart.process_tick(&tick);
                    ticks_processed_by_charts += 1;
                }

                if first_tick {
                    first_tick = false;
                    current_day_number = tick.time.tm_mday;
                } else if current_day_number != tick.time.tm_mday {
                    current_day_number = tick.time.tm_mday;
                    num_days += 1;

                    if 0 == num_days % 6 {
                        num_days += 1; // no tick data on saturdays, so skip it
                    }

                    if num_days >= target_num_days {
                        // println!("Walked to {}", utilities::tm_to_iso(tick.time));
                        break;
                    }
                }
            }

            charts = pristine_charts.clone();
        }

        // ----- GENERATE IN SAMPLE TICKS ----------------------------------------------------------

        println!("==================== GENERATING IN SAMPLE #{} ====================", i);
        // println!("Next tick: {}", utilities::tm_to_iso(Tick::new_from_line(file.read_line().ok().unwrap()).time));

        first_tick = true;
        num_days = 0i32;
        current_day_number = 0i32;
        target_num_days = in_sample_days;
        ticks = vec!();

        bytes_read_at_in_sample = bytes_read;
        // println!("Recording current file cursor: {}", bytes_read_at_in_sample);

        // println!("Advancing {} days", in_sample_days);

        for mut line in file.lines().filter_map( |l| l.ok() ) {
            bytes_read += line.len();

            let tick = Tick::new_from_line(line);

            if first_tick {
                first_tick = false;
                current_day_number = tick.time.tm_mday;
                // println!("FIRST TICK: {}", utilities::tm_to_iso(tick.time));
            } else if current_day_number != tick.time.tm_mday {
                current_day_number = tick.time.tm_mday;
                num_days += 1;

                if 0 == num_days % 6 {
                    num_days += 1; // no tick data on saturdays, so skip it
                }

                if num_days >= target_num_days {
                    // println!("last tick: {}", utilities::tm_to_iso(tick.time));
                    break;
                }
            }

            ticks.push(tick);
        }

        bytes_read_at_out_of_sample = bytes_read;

        let in_begin_tick = ticks[0].time;
        let in_end_tick   = ticks[ticks.len()-1].time;

        println!(
            "IN SAMPLE: {} - {} ({} ticks)",
            utilities::tm_to_iso(in_begin_tick),
            utilities::tm_to_iso(in_end_tick),
            ticks.len(),
        );

        // ----- OPTIMIZE ON IN SAMPLE TICKS -------------------------------------------------------

        println!("==================== OPTIMIZING ====================");
        // println!("Next tick: {}", utilities::tm_to_iso(Tick::new_from_line(file.read_line().ok().unwrap()).time));

        let vars = match optimizer.variables_for(charts.clone(), &ticks, &mut trades_log, &mut ticks_log) {
            Some(vars) => vars,
            None       => {
                failed_to_optimize_algorithm = true;
                break
            },
        };

        // ----- WALK CHARTS FORWARD TO BEGINNING OF OUT OF SAMPLE PERIOD --------------------------

        println!("==================== APPLYING IN SAMPLE TO CHARTS ====================");

        // println!("Rewinding to stored cursor at beginning of in sample period");
        // println!("next line: {}", file.read_line().ok().unwrap());
        file = utilities::buf_reader_from_file(csv_path, bytes_read_at_in_sample);
        bytes_read = bytes_read_at_in_sample;

        // println!("Bytes read: {}", bytes_read);
        // println!("Bytes read at out of sample: {}", bytes_read_at_out_of_sample);

        for mut line in file.lines().filter_map( |l| l.ok() ) {
            bytes_read += line.len();

            let tick = Tick::new_from_line(line);

            for chart in charts.iter_mut() {
                chart.process_tick(&tick);
            }

            if bytes_read == bytes_read_at_out_of_sample {
                break;
            }
        }

        // ----- GENERATE OUT OF SAMPLE TICKS ------------------------------------------------------

        println!("==================== GENERATING OUT OF SAMPLE #{} ====================", i);
        first_tick = true;
        num_days = 0;
        target_num_days = out_of_sample_days;
        ticks = vec!();

        for mut tick in file.lines().filter_map( |l| l.ok() ).map( |l| Tick::new_from_line(l) ) {
            if first_tick {
                first_tick = false;
                current_day_number = tick.time.tm_mday;
                // println!("FIRST TICK: {}", utilities::tm_to_iso(tick.time));
            } else if current_day_number != tick.time.tm_mday {
                current_day_number = tick.time.tm_mday;
                num_days += 1;

                if 0 == num_days % 6 {
                    num_days += 1; // no tick data on saturdays, so skip it
                }

                if num_days >= target_num_days {
                    break;
                }
            }

            ticks.push(tick);
        }

        let out_begin_tick = ticks[0].time;
        let out_end_tick   = ticks[ticks.len()-1].time;

        println!(
            "OUT OF SAMPLE: {} - {} ({} ticks)",
            utilities::tm_to_iso(out_begin_tick),
            utilities::tm_to_iso(out_end_tick),
            ticks.len(),
        );

        // ----- EXECUTE ON OUT OF SAMPLE TICKS ----------------------------------------------------

        println!("==================== EXECUTING ====================");

        let strat = strategy.clone();
        let mut algorithm = Algorithm::new_out_of_sample(strat, charts.clone());
        let score = match algorithm.execute_on(&ticks, vars.clone(), &mut trades_log, &mut ticks_log) {
            Some(score) => score,
            None        => {
                failed_to_execute = true;
                break
            },
        };

        scores.push(score);
    }

    println!("CHARTS");
    for chart in pristine_charts.iter() {
        println!("Charts processed {} ticks", chart.get_ticks_processed());
    }

    println!("ticks_processed_by_charts: {}", ticks_processed_by_charts);

    if failed_to_optimize_algorithm {
        println!("Algorithm failed on in sample optimization");
    } else if failed_to_execute {
        println!("Algorithm failed on out of sample execution");
    } else {
        println!("SCORES:");

        let mut count = 1i32;

        for score in scores.iter() {
            println!("{}: {}", count, score);

            count += 1;
        }
    }

    // ----- POST RUN SCRIPT -----------------------------------------------------------------------

    if post_run_script.len() > 0 {
        println!("");
        println!("==================== POST-RUN PROCESSING ====================");
        println!("");
        println!("Executing: {}", post_run_script);
        println!("");

        let output = match Command::new("bash").arg("-c").arg(post_run_script).output() {
            Ok(output) => output,
            Err(e)     => panic!("Failed to execute post-run script: {}", e),
        };

        println!("Post-run script exited with {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(output.output.as_slice()));
        println!("stderr: {}", String::from_utf8_lossy(output.error.as_slice()));
    }
}
