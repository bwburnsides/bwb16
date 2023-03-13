struct Cpu {
    _registers: [u16; 16],
    _carry: bool,
    _zero: bool,
    _negative: bool,
    _overflow: bool,
    _kernel: bool,
}

impl Cpu {
    fn new() -> Self {
        Self {
            _registers: [0; 16],
            _carry: false,
            _zero: false,
            _negative: false,
            _overflow: false,
            _kernel: true,
        }
    }

    fn _run(&self) {}
}

fn main() {
    let _cpu = Cpu::new();
}
