extern crate common;

use common::*;

#[no_mangle]
pub fn new_state() -> State {
    //Demo:
    //1. Run `cargo run` in the folder containing the `state_manipulation` folder
    //   Leave the windoe open.
    //2. Change this string and save the file.
    //3. Run `cargo build` in the `state_manipulation` folder.
    //4. See that the string has changed in the running  program!
    State { greeting: "Hello World!".to_string() }
}

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, game: &mut State, events: &mut Vec<Event>) -> bool {

    for event in events {
        match *event {
            Event::Close |
            Event::KeyPressed { key: KeyCode::Escape, ctrl: _, shift: _ } => return true,
            _ => (),
        }
    }

    draw(platform, game);

    false
}

fn draw(platform: &Platform, state: &State) {
    (platform.print_xy)(34, 14, &state.greeting);
}
