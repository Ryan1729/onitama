extern crate common;

use common::*;

#[no_mangle]
pub fn new_state() -> State {
    State { x: 0 }
}

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {

    for event in events {
        match *event {
            Event::Close |
            Event::KeyPressed { key: KeyCode::Escape, ctrl: _, shift: _ } => return true,
            _ => (),
        }
    }

    state.x += 1;
    state.x %= 80;

    draw(platform, state);

    false
}

fn draw(platform: &Platform, state: &State) {
    //Demo:
    //1. Run `cargo run` in the folder containing the `state_manipulation` folder
    //   Leave the windoe open.
    //2. Change this string and save the file.
    //3. Run `cargo build` in the `state_manipulation` folder.
    //4. See that the string has changed in the running  program!
    (platform.print_xy)(34, 14, "Hello World!");

    (platform.print_xy)(state.x, 15, "‾");
}
