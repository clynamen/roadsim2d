use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use std::vec::Vec;
use std::collections::hash_set::HashSet;
use std::collections::vec_deque::VecDeque;
extern crate piston_window;

// credits to https://github.com/andreivasiliu/stacked-worlds on how to handle specs and input

pub enum InputEvent {
    PressEvent(piston_window::Button),
    ReleaseEvent(piston_window::Button),
    MotionEvent(f64, f64),
}


pub struct InputEvents {
    pub events: VecDeque<InputEvent>,
}

impl InputEvents {
    pub fn new() -> Self {
        InputEvents {
            events: VecDeque::with_capacity(32),
        }
    }
}

#[derive(Debug)]
pub struct InputState {
    pub buttons_pre_pressed: HashSet<piston_window::Button>,
    pub buttons_post_pressed: HashSet<piston_window::Button>,
    pub buttons_pressed: HashSet<piston_window::Button>,
    pub buttons_held: HashSet<piston_window::Button>
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            buttons_pre_pressed: HashSet::new(),
            buttons_post_pressed: HashSet::new(),
            buttons_pressed: HashSet::new(),
            buttons_held: HashSet::new(),
        }
    }

    pub fn button_press(&mut self, button: piston_window::Button) {
        self.buttons_pre_pressed.insert(button);
        self.buttons_held.insert(button);
    }

    pub fn button_release(&mut self, button: piston_window::Button) {
        self.buttons_post_pressed.remove(&button);
        self.buttons_held.remove(&button);
    }

}


pub struct UpdateInputStateSys;

impl <'a> System<'a> for UpdateInputStateSys {
    type SystemData = (
        WriteExpect<'a, InputState>,
    );

    fn run(&mut self, (mut input_state, ): Self::SystemData) {
        let mut new_pressed_buttons = Vec::<piston_window::Button>::new();
        let mut old_pressed_buttons = Vec::<piston_window::Button>::new();

        for button in &input_state.buttons_pre_pressed {
            if !input_state.buttons_post_pressed.contains(&button) {
                new_pressed_buttons.push(button.clone());
            }
        }

        for button in &input_state.buttons_pressed {
            old_pressed_buttons.push(button.clone());
        }
        for button in old_pressed_buttons {
            input_state.buttons_post_pressed.insert(button);
        }

        input_state.buttons_pre_pressed.clear();
        input_state.buttons_pressed.clear();

        for button in new_pressed_buttons {
            input_state.buttons_pressed.insert(button);
        }

    }
}

pub struct HandleInputEventSys;


impl <'a> System<'a> for HandleInputEventSys {
    type SystemData = (
        WriteExpect<'a, InputEvents>,
        WriteExpect<'a, InputState>,
    );

    fn run(&mut self, (mut input_events, mut input_state): Self::SystemData) {
        while let Some(input_event) = input_events.events.pop_front() {
            match input_event {
                InputEvent::PressEvent(button) => {
                    input_state.button_press(button);
                },
                InputEvent::ReleaseEvent(button) => {
                    input_state.button_release(button);

                },
                InputEvent::MotionEvent(x, y) => {

                },
            };

        }
    }
}