use std::io::{BufferedReader,File,MemReader};

use chart::Chart;
use indicators::Indicator;
use range_bound_variable::RangeBoundVariables;
use parser_utils;
use utilities;

// ===== CHARTS ====================================================================================

fn parse_indicator(lua_chart_name: &str,
                   indicator_string: &str,
                   num_chart_candles: i32) -> Indicator {
    let indicator_parts = utilities::split_csv_string(indicator_string, ',');

    if 2 != indicator_parts.len() {
        panic!("indicator definition must have 2 parts");
    }

    let indicator_type        = indicator_parts[0];
    let num_indicator_candles = utilities::string_to_int(indicator_parts[1]) as i32;

    if num_indicator_candles < 1 {
        panic!("number of candles for indicator must be > 0");
    } else if num_indicator_candles > num_chart_candles {
        panic!(
            "number of indicator candles ({}) can't exceed number of chart candles ({})",
            num_chart_candles,
            num_indicator_candles,
        );
    }

    // e.g., "candlestick_M1_sma_60" from the example above
    let lua_indicator_name = format!("{}_{}_{}", lua_chart_name, indicator_type, num_indicator_candles);
    parser_utils::validate_name(lua_indicator_name.as_slice());

    let indi = match indicator_type {
        "sma" => {
            Indicator::new_sma(lua_indicator_name.as_slice(), num_indicator_candles)
        },
        _     => panic!("unknown indicator type: {}", indicator_type)
    };

    println!("Loaded indicator {}", lua_indicator_name);

    indi
}

fn parse_charts<T: Buffer>(buffer: &mut T) -> Vec<Chart> {
    let mut charts: Vec<Chart> = vec!();
    let mut num_loaded = 0i32;

    for mut line in buffer.lines().filter_map( |result| result.ok() ) {
        let old_len = line.len();
        let length = old_len - 1;
        line.truncate(length);

        // skip empty lines and comment lines
        if parser_utils::empty_or_comment(line.as_slice()) {
            continue;
        }

        // Example line: candlestick,M1,60|sma,12
        // Chart is "type,period,num_candles".  Lua variable is just "#{type}_#{period}",
        // e.g., "candlestick_M1".
        // Indicators follow the first pipe.  Format is "type,num_candles".  Lua variable is
        // "#{chart_type}_#{chart_period}_#{indicator_name}_#{num_indicator_candles}"

        let parts = utilities::split_csv_string(line.as_slice(), '|');

        if parts.len() < 2 {
            panic!("line must have > 2 parts");
        }

        let chart_section     = parts[0];
        let indicator_section = parts[1];

        // ----- CREATE CHART ----------------------------------------------------------------------

        let chart_parts = utilities::split_csv_string(chart_section, ',');

        if 3 != chart_parts.len() {
            panic!("chart definition must have 3 parts")
        }

        let chart_type  = chart_parts[0];
        let period      = chart_parts[1];
        let num_chart_candles = utilities::string_to_int(chart_parts[2]) as i32;

        if num_chart_candles < 1 {
            panic!("number of candles for chart must be > 0");
        }

        let lua_chart_name = format!("{}_{}", chart_type, period);
        parser_utils::validate_name(lua_chart_name.as_slice());

        let mut chart = match chart_type {
            "candlestick" => Chart::new_candlestick_chart(lua_chart_name.as_slice(), period, num_chart_candles),
            _             => panic!("unknown chart type: {}", chart_type),
        };

        println!("Loaded chart {}", lua_chart_name);

        // ----- CREATE AND ATTACH INDICATORS ------------------------------------------------------

        for &indicator_string in utilities::split_csv_string(indicator_section, ':').iter() {
            let x = parse_indicator(lua_chart_name.as_slice(), indicator_string, num_chart_candles);
            chart.attach_indicator(x);
        }

        // ----- APPEND CHART ----------------------------------------------------------------------

        charts.push(chart);

        num_loaded += 1;
    }

    println!("Loaded {} charts", num_loaded);

    charts
}

pub fn parse_charts_from_file(path: &str) -> Vec<Chart> {
    let fd = match File::open(&Path::new(path)).ok() {
        Some(val) => val,
        None      => panic!("can't open file at path: {}", path)
    };
    let mut reader = BufferedReader::new(fd);

    println!("Loading charts and indicators from {}", path);

    parse_charts(&mut reader)
}

pub fn parse_charts_from_string(s: String) -> Vec<Chart> {
    let mut reader = MemReader::new(s.into_bytes());

    println!("Loading charts and indicators from string");

    parse_charts(&mut reader)
}

// ===== RANGE-BOUND VARIABLES =====================================================================

fn parse_variables<T: Buffer>(buffer: &mut T) -> RangeBoundVariables {
    let mut rbv = RangeBoundVariables::new();
    let mut num_loaded = 0i32;

    for mut line in buffer.lines().filter_map( |result| result.ok() ) {
        let old_len = line.len();
        let length = old_len - 1;
        line.truncate(length);

        // skip empty lines and comment lines
        if parser_utils::empty_or_comment(line.as_slice()) {
            continue;
        }

        let parts: Vec<&str> = line.as_slice().split(',').collect();
        let name = parts[0];
        let var_type = parts[1];

        parser_utils::validate_name(name);

        match var_type {
            "bool"  => rbv.create_bool(name),
            "float" => {
                let lower_bound = utilities::string_to_float(parts[2]);
                let upper_bound = utilities::string_to_float(parts[3]);
                rbv.create_float(name, lower_bound, upper_bound)
            },
            "int"   => {
                let lower_bound = utilities::string_to_int(parts[2]);
                let upper_bound = utilities::string_to_int(parts[3]);
                rbv.create_int(name, lower_bound, upper_bound)
            },
            _       => panic!("unknown variable type: {}", var_type)
        };

        num_loaded += 1;
    }

    println!("Loaded {} variables", num_loaded);

    rbv
}

pub fn parse_variables_from_file(path: &str) -> RangeBoundVariables {
    let fd = match File::open(&Path::new(path)).ok() {
        Some(val) => val,
        None      => panic!("can't open file at path: {}", path)
    };
    let mut reader = BufferedReader::new(fd);

    println!("Loading variables from {}", path);

    parse_variables(&mut reader)
}

pub fn parse_variables_from_string(s: String) -> RangeBoundVariables {
    let mut reader = MemReader::new(s.into_bytes());

    println!("Loading variables from string");

    parse_variables(&mut reader)
}
