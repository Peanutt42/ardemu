# Rust AVR Fibonacci

Rust nightly required.

```
RUSTFLAGS="-C target-cpu=atmega328p" cargo build --target avr-none -Z build-std=core --release
```

The final ELF executable file will then be available at `PROJECT_ROOT/target/avr-none/release/rust_fib.elf`.

