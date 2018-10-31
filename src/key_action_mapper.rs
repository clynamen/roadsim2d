use std::boxed::Box;
use std::collections::HashSet;
use std::time;
use super::debouncer::*;
use std::collections::HashMap;

pub struct KeyActionMapper<T> {
    key_action_map: HashMap<piston_window::Key, Box<Debouncer<T>>>
}

impl<T> KeyActionMapper<T> {
    pub fn new() -> KeyActionMapper<T> {
        let mut key_action_map: HashMap<
            piston_window::Key,
            Box<Debouncer<T>>,
        > = HashMap::new();


        KeyActionMapper::<T> {
            key_action_map: key_action_map
        }
    }

    pub fn add_action<F>(&mut self, key: piston_window::Key, debounce_time_ms: u64, fun: F)
         where F: for <'a> FnMut(&'a mut T) + 'static  {
        let debouncer: Debouncer<T> =
            Debouncer::from_millis(debounce_time_ms, fun);
        let debouncer_box = Box::new(debouncer);
        self.key_action_map.insert(key, debouncer_box);
    }

    pub fn process_buttons(&mut self, buttons: &HashSet<piston_window::Button>, obj: &mut T) {
        for button in buttons {
            match button {
                piston_window::Button::Keyboard(key) =>  {
                    let action = self.key_action_map.get_mut(key);
                    if action.is_some() {
                        action.unwrap().debounce(obj);
                    }
                }
                _ => {}
            }
        }
    }

}