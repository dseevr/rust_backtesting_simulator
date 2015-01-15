use std::collections::hash_map;
use std::rand;
use std::rand::{thread_rng, Rng};

use lua;
use parsers;

// ----- BOOL --------------------------------------------------------------------------------------

#[derive(Clone,Show)]
struct RangeBoundBool {
    value: bool,
}

impl RangeBoundBool {
    fn new() -> RangeBoundBool {
        RangeBoundBool{ value: false }
    }

    fn randomize(&mut self) {
        self.value = rand::thread_rng().gen();
    }

    fn value(&self) -> bool {
        self.value
    }
}

// ----- FLOAT -------------------------------------------------------------------------------------

#[derive(Clone,Show)]
struct RangeBoundFloat {
    lower: f32,
    upper: f32,
    value: f32,
}

impl RangeBoundFloat {
    fn new(lower: f32, upper: f32) -> RangeBoundFloat {
        if lower >= upper {
            panic!("lower ({}) must be < upper ({})", lower, upper);
        }

        RangeBoundFloat { lower: lower, upper: upper, value: 0.0f32 }
    }

    fn randomize(&mut self) {
        // gen range is exclusive of the upper bound, but we don't care because it's a float
        self.value = rand::thread_rng().gen_range(self.lower, self.upper);
    }

    fn value(&self) -> f32 {
        self.value
    }
}

// ----- INTEGER -----------------------------------------------------------------------------------

#[derive(Clone,Show)]
struct RangeBoundInteger {
    lower: i32,
    upper: i32,
    value: i32,
}

impl RangeBoundInteger {
    fn new(lower: i32, upper: i32) -> RangeBoundInteger {
        if lower >= upper {
            panic!("lower ({}) must be < upper ({})", lower, upper);
        }

        RangeBoundInteger { lower: lower, upper: upper, value: 0i32 }
    }

    fn randomize(&mut self) {
        // gen_range is exclusive of the upper bound
        self.value = rand::thread_rng().gen_range(self.lower, self.upper + 1);
    }

    fn value(&self) -> i32 {
        self.value
    }
}

// ----- VARIABLES ---------------------------------------------------------------------------------

#[derive(Clone,Show)]
pub struct RangeBoundVariables {
    bools:  hash_map::HashMap<String, RangeBoundBool>,
    floats: hash_map::HashMap<String, RangeBoundFloat>,
    ints:   hash_map::HashMap<String, RangeBoundInteger>,
}

impl RangeBoundVariables {
    pub fn new() -> RangeBoundVariables {
        let bools:  hash_map::HashMap<String, RangeBoundBool>    = hash_map::HashMap::new();
        let floats: hash_map::HashMap<String, RangeBoundFloat>   = hash_map::HashMap::new();
        let ints:   hash_map::HashMap<String, RangeBoundInteger> = hash_map::HashMap::new();

        RangeBoundVariables { bools: bools, floats: floats, ints: ints }
    }

    pub fn new_from_file(path: &str) -> RangeBoundVariables {
        parsers::parse_variables_from_file(path)
    }

    pub fn new_from_string(text: String) -> RangeBoundVariables {
        parsers::parse_variables_from_string(text)
    }

    pub fn create_bool(&mut self, name: &str) {
        if self.bools.contains_key(name) {
            panic!("a bool already exists with the name \"{}\"", name);
        }

        let key = name.clone();
        let val = RangeBoundBool::new();
        self.bools.insert(key.to_string(), val);
    }

    pub fn create_float(&mut self, name: &str, lower: f32, upper: f32) {
        if self.floats.contains_key(name) {
            panic!("a float already exists with the name \"{}\"", name);
        }

        let key = name.clone();
        let val = RangeBoundFloat::new(lower, upper);
        self.floats.insert(key.to_string(), val);
    }

    pub fn create_int(&mut self, name: &str, lower: i32, upper: i32) {
        if self.ints.contains_key(name) {
            panic!("an integer already exists with the name \"{}\"", name);
        }

        let key = name.clone();
        let val = RangeBoundInteger::new(lower, upper);
        self.ints.insert(key.to_string(), val);
    }

    pub fn get_bool(&self, name: &str) -> bool {
        match self.bools.get(name) {
            Some(x) => x.value(),
            None    => panic!("could not find a bool with the name \"{}\"", name)
        }
    }

    pub fn get_float(&self, name: &str) -> f32 {
        match self.floats.get(name) {
            Some(x) => x.value(),
            None    => panic!("could not find a float with the name \"{}\"", name)
        }
    }

    pub fn get_integer(&self, name: &str) -> i32 {
        match self.ints.get(name) {
            Some(x) => x.value(),
            None    => panic!("could not find an integer with the name \"{}\"", name)
        }
    }

    pub fn print(&self) {
        for (name, value) in self.bools.iter() {
            println!("{} => {}", name, value.value());
        }

        for (name, value) in self.floats.iter() {
            println!("{} => {}", name, value.value());
        }

        for (name, value) in self.ints.iter() {
            println!("{} => {}", name, value.value());
        }
    }

    pub fn randomize(&mut self) {
        for (_, value) in self.bools.iter_mut() {
            value.randomize();
        }

        for (_, value) in self.floats.iter_mut() {
            value.randomize();
        }

        for (_, value) in self.ints.iter_mut() {
            value.randomize();
        }
    }

    pub fn register_in_lua(&self) {
        for (name, value) in self.bools.iter() {
            lua::register_boolean(name.as_slice(), value.value());
        }

        for (name, value) in self.floats.iter() {
            lua::register_number(name.as_slice(), value.value());
        }

        for (name, value) in self.ints.iter() {
            lua::register_number(name.as_slice(), value.value() as f32);
        }
    }
}
