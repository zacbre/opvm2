const MAX_HEAP_SIZE: usize = 1024;

pub struct Heap {
    data: [u8; MAX_HEAP_SIZE],
}

impl Heap {
    pub fn new() -> Self {
        Self { data: [0; MAX_HEAP_SIZE] }
    }

    // find the next block that is available
    pub fn allocate() {

    }
}