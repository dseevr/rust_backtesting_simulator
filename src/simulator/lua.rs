extern crate libc;

use self::libc::{c_char,c_int,c_float};
// use std::c_str::ToCStr;
use std::ffi::CString;
use std::ffi;
use std::str;

#[link(name = "bridge")]
extern {
    // interpreter functions
    fn lua_bridge_setup(path: *const libc::c_char);
    fn lua_bridge_teardown();

    // config file functions
    // fn lua_bridge_open_config(path: *const libc::c_char);

    // variable functions
    fn lua_bridge_get_string_var(name: *const libc::c_char) -> *const libc::c_char;
    fn lua_bridge_get_int_var(name: *const libc::c_char) -> libc::c_int;
    fn lua_bridge_register_string(name: *const libc::c_char, value: *const libc::c_char);
    fn lua_bridge_register_number(name: *const libc::c_char, value: libc::c_float);
    fn lua_bridge_register_boolean(name: *const libc::c_char, value: libc::c_int);

    // trading functions
    fn lua_get_decision() -> libc::c_int; // TODO: rename so it matches the rest
    fn lua_bridge_on_tick();

    // chart functions
    fn lua_bridge_create_table(size: libc::c_int);
    fn lua_bridge_push_table_integer(num: libc::c_int);
    fn lua_bridge_push_table_number(num: libc::c_float);
    fn lua_bridge_push_table_string(name: *const libc::c_char);
    fn lua_bridge_set_table(offset: libc::c_int);
    fn lua_bridge_finalize_table(name: *const libc::c_char);
}

// ===== INTERPRETER FUNCTIONS =====================================================================

pub fn setup(path: &str) {
    println!("Starting Lua interpreter with script {}", path);

    unsafe {
        lua_bridge_setup(path.to_c_str().as_ptr());
    }
}

pub fn teardown() {
    println!("Stopping Lua interpeter");

    unsafe {
        lua_bridge_teardown();
    }
}

// ===== CHART FUNCTIONS ===========================================================================

pub fn create_table(size: i32) {
    // println!("new lua table with size: {}", size);
    unsafe {
        lua_bridge_create_table(size);
    }
}

pub fn push_table_integer(num: i32) {
    // println!("pushed integer: {}", num);
    unsafe {
        lua_bridge_push_table_integer(num);
    }
}

pub fn push_table_number(num: f32) {
    // println!("pushed number: {}", num);
    unsafe {
        lua_bridge_push_table_number(num);
    }
}

pub fn push_table_string(s: &str) {
    // println!("pushed string: {}", s);
    unsafe {
        lua_bridge_push_table_string(s.to_c_str().as_ptr());
    }
}

pub fn set_table(offset: i32) {
    // println!("pushed table with offset: {}", offset);
    unsafe {
        lua_bridge_set_table(offset);
    }
}

pub fn finalize_table(name: &str) {
    // println!("finalizing table with name: {}", name);
    unsafe {
        lua_bridge_finalize_table(name.to_c_str().as_ptr());
    }
}

// ===== VARIABLE FUNCTIONS ========================================================================

    pub fn get_string_var(name: &str) -> String {
        // *const libc::c_char
        let s = unsafe {
            let c_ptr = lua_bridge_get_string_var(name.to_c_str().as_ptr());
            let slice = ffi::c_str_to_bytes(&c_ptr);
            str::from_utf8(slice).unwrap()
            // CString::new(c_ptr, false)  // false because you don't own this string, it is static
        };
    
        let s_slice: &str = s.as_str().unwrap().clone();
    
        // s_slice.to_string()
    }

pub fn get_int_var(name: &str) -> i32 {
    unsafe {
        lua_bridge_get_int_var(name.to_c_str().as_ptr())
    }
}

pub fn register_string(name: &str, value: &str) {
    unsafe {
        lua_bridge_register_string(name.to_c_str().as_ptr(), value.to_c_str().as_ptr());
    }
}

pub fn register_number(name: &str, value: f32) {
    unsafe {
        lua_bridge_register_number(name.to_c_str().as_ptr(), value);
    }
}

pub fn register_boolean(name: &str, value: bool) {
    let int_form = match value {
        true  => 1,
        false => 0,
    };

    unsafe {
        lua_bridge_register_boolean(name.to_c_str().as_ptr(), int_form);
    }
}

// ===== TRADING FUNCTIONS =========================================================================

#[derive(Copy)]
pub enum TradeDecision {
    NOOP,
    LONG,
    SHORT,
    CLOSE,
}

pub fn on_tick() -> TradeDecision {
    unsafe {
        lua_bridge_on_tick();
    }

    // enum DECISION {
    //     NOOP = 0,
    //     LONG = 1,
    //     SHORT = 2,
    //     CLOSE = 3
    // } trading_decisions;

    let decision = get_decision();

    match decision {
        0 => TradeDecision::NOOP,
        1 => TradeDecision::LONG,
        2 => TradeDecision::SHORT,
        3 => TradeDecision::CLOSE,
        _ => panic!("unknown value: {}", decision),
    }
}

fn get_decision() -> i32 {
    unsafe {
        lua_get_decision()
    }
}
