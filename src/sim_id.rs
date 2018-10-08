

pub struct IdProvider {
    last_id : u64,
}

impl IdProvider {
    pub fn new() -> IdProvider {
        IdProvider { 
            last_id: 0u64
        }
    }
    pub fn next(&mut self) -> u64 {
        let next_id = self.last_id;
        self.last_id += 1;
        next_id
    }

}