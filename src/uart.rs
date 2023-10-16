pub struct Uart {
    ptr: *mut u8,
}

impl Uart {
    /// # Safety
    ///
    /// The base address needs to point to a valid UART device
    pub unsafe fn new(base_address: usize) -> Self {
        Self {
            ptr: base_address as *mut u8,
        }
    }

    pub fn init(&self) {
        // Safety: self.ptr points to a valid UART device
        unsafe {
            // Set word lenght to 8
            self.ptr.add(3).write_volatile(0b11);

            // Enable FIFO
            self.ptr.add(2).write_volatile(0b1);

            // Enable interrupts
            self.ptr.add(1).write_volatile(0b1);

            // Set baud rate
            let divisor: u16 = 592;
            let divisor_low: u8 = (divisor & 0xFF) as u8;
            let divisor_high: u8 = (divisor >> 8) as u8;

            self.ptr.add(3).write_volatile(0x11 | (1 << 7));

            self.ptr.add(0).write_volatile(divisor_low);
            self.ptr.add(0).write_volatile(divisor_high);

            self.ptr.add(3).write_volatile(0x11);
        }
    }

    pub fn get(&self) -> Option<u8> {
        // Safety: self.ptr points to a valid UART device
        unsafe {
            if self.ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(self.ptr.add(0).read_volatile())
            }
        }
    }

    pub fn get_blocking(&self) -> u8 {
        // TODO: Wait for interrupts instead of pooling
        loop {
            if let Some(c) = self.get() {
                break c;
            }
        }
    }

    pub fn put(&self, c: u8) {
        // Safety: self.ptr points to a valid UART device
        unsafe {
            self.ptr.add(0).write_volatile(c);
        }
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}
