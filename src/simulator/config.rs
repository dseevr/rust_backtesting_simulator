use std::mem;

use lua;

static mut CONFIG: *mut ConfigurationFile = 0 as *mut ConfigurationFile;

pub struct ConfigurationFile {
    pub csv_path: String,

    pub charts: String,
    pub variables: String,

    pub in_sample:     String,
    pub out_of_sample: String,

    pub iterations: i32,
    pub steps:      i32,

    pub jpy_base: bool,

    pub post_run_script: String,
}

pub fn get<'a>() -> &'a mut ConfigurationFile {
    unsafe {
        mem::transmute(CONFIG)
    }
}

impl ConfigurationFile {
    pub fn load_from_file(path: &str) {
        lua::setup(path);

        let csv_path = lua::get_string_var("CSV_PATH");

        let in_sample     = lua::get_string_var("IN_SAMPLE_TIME");
        let out_of_sample = lua::get_string_var("OUT_OF_SAMPLE_TIME");

        let iterations = lua::get_int_var("ITERATIONS");
        let steps      = lua::get_int_var("STEPS");

        let charts     = lua::get_string_var("CHARTS");
        let variables  = lua::get_string_var("VARIABLES");

        let jpy_base = false; // is set later

        let post_run_script = lua::get_string_var("POST_RUN_SCRIPT");

        lua::teardown();

        if steps < 1 {
            panic!("STEPS must be > 0");
        }

        let config = ConfigurationFile {
            charts: charts,
            csv_path: csv_path,
            in_sample: in_sample,
            out_of_sample: out_of_sample,
            variables: variables,
            iterations: iterations,
            steps: steps,
            jpy_base: jpy_base,
            post_run_script: post_run_script,
        };

        unsafe {
            let box_config = Box::new(config);
            CONFIG = mem::transmute(box_config);
        }
    }

    pub fn set_jpy_base(&mut self) {
        self.jpy_base = true;
    }
}
