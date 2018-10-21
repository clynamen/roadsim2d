extern crate floating_duration;

use std::time::{Instant, Duration};

pub struct Debouncer<A> {
    min_dt: Duration,
    last_t: Instant,
    fun: Box<for <'a> FnMut(&'a mut A)>,
    _phantom : std::marker::PhantomData<A>
}

// fn call_twice<A, F>(val: A, mut f: F) -> A
// where F: FnMut(A) -> A {
//     let tmp = f(val);
//     f(tmp)
// }

impl<A> Debouncer<A> {
    // pub fn from_millis<A> (ms: u64, fun: A) -> Debouncer<A> where A: FnMut() -> ()  {
    pub fn from_millis<F>(ms: u64, fun: F) -> Debouncer<A> where F: for <'a> FnMut(&'a mut A) + 'static { 
        Debouncer {
            min_dt: Duration::from_millis(ms),
            last_t: Instant::now(),
            fun: Box::new(fun),
            _phantom : std::marker::PhantomData
        }
    }

    pub fn debounce<'a>(&mut self, obj: &'a mut A) {
        let now = Instant::now();
        let diff = now.duration_since(self.last_t);
        if diff >  self.min_dt {
            self.last_t = now;
            (self.fun)(obj);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constructor() {
        let debouncer = Debouncer::from_millis(100, {println!("hello world")});
    }


}