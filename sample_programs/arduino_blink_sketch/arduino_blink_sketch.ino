void setup() {
  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);
  interrupt_free_delay(1000);
  digitalWrite(LED_BUILTIN, LOW);
  interrupt_free_delay(1000);
}

void interrupt_free_delay(uint16_t millis) {
  for (uint16_t i = 0; i < millis; i++) {
    // 1000 mircosec = 1 ms
    interrupt_free_delay_1000_micros();
  }
}

void interrupt_free_delay_1000_micros() {
    const uint16_t cycles_per_microsec = F_CPU / 1000000;
    const uint16_t micros = 1000;
    const uint16_t cycles = micros * cycles_per_microsec;
    // nop: 1 cycle
    // subi: 1 cycle
    // sbc: 1 cycle
    // brne: 2 cycle (if branching), 1 cycle (if not branching)
    //    -> most of the time: 2 cycles
    // => Total: 5 cycles
    const uint16_t cycles_per_loop = 5;
    const uint16_t loops = cycles / cycles_per_loop;


    for (uint16_t i = 0; i < loops; i++) {
      asm volatile ("nop \n\t");
    }
}
