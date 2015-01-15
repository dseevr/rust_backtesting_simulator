use std::io::Command;
use std::os;

fn main() {
    let out_dir = os::getenv("OUT_DIR").unwrap();

    Command::new("gcc-4.9").args(&["bridge/bridge.c", "-c", "-o"])
                       .arg(format!("{}/bridge.o", out_dir))
                       .status().unwrap();
    Command::new("ar").args(&["crus", "libbridge.a", "bridge.o"])
                      .cwd(&Path::new(&out_dir))
                      .status().unwrap();

    // Command::new("gcc").args(&["-static", "-shared bridge.o", "-L/Users/bill/src/lua-5.2.3/src/",
    //                   "-I/Users/bill/src/lua-5.2.3", "-llua", "-lm", "-ldl"])
    //                   .arg(format!("-o {}/bridge.lib", out_dir))
    //                   .cwd(&Path::new(&out_dir)).status().unwrap();

    println!("cargo:rustc-flags=-L {} -l bridge -l lua", out_dir);
}
