#![no_std]
#![cfg_attr(not(test), no_main)]

extern crate avr_std_stub;

#[cfg(not(test))]
fn fib(n: u8) -> u8 {
	if n == 0 || n == 1 {
		n
	} else {
		fib(n - 1) + fib(n - 2)
	}
}

#[no_mangle]
#[cfg(not(test))]
fn main() -> i32 {
	fib(10) as i32
}
