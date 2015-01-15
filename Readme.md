# Rust simulator

These 103 commits were written between 2014-12-13 and 2015-01-07.

This was my third attempt at building a simulator, and the first greenfield attempt at implementing **Walk Forward Optimization**.  This simulator also brings in Lua scripting for setting the simulation config and implementing the trading algorithm.

I got all the "walk forward" stuff working in main.rs (not refactored and pretty ugly), but then a new nightly Rust build moved a lot of stuff around and the docs weren't updated.  I was also having serious concerns about the lack of ability to have a Vec of box'd Traits easily stored on a struct.  I ended up having to do crazy things to implement what is simply done using an abstract virtual base class in C++.

I decided to port all of this pared down code to C++14 and save myself the headaches.  C++ means I can also use LuaJIT which I was unable to get working with Rust due to its odd build system.

## Usage

This code is not working with the latest nightly builds as of 2015-01-08.  It probably requires small changes to get back up to speed as Rust approaches its 1.0 alpha release.

## License

BSD
