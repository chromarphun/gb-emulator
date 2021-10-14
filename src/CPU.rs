pub struct CentralProcessingUnit {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    pc: u16,
    sp: u16,
    memory_mut: Arc<Mutex<[u8; 65536]>>,
}

impl CentralProcessingUnit {
    pub fn new(memory_mut: Arc<Mutex<[u8; 65536]>>) -> CentralProcessingUnit {
        let af = 0;
        let bc = 0;
        let de = 0
        let hl = 0
        let pc = 0x100
        let sp = 0xFFFE
        CentralProcessingUnit {
            af,
            bc,
            de,
            hl,
            pc,
            sp,
            memory_mut,
        }
    }

}