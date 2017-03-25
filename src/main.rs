extern crate libloading;
extern crate bear_lib_terminal;
extern crate common;

use libloading::Library;

use bear_lib_terminal::terminal::{self, config, Event, KeyCode, state};
use bear_lib_terminal::Color;
use bear_lib_terminal::geometry::{Point, Rect, Size};

use std::mem;

use common::*;

const LIB_PATH: &'static str = "./target/debug/libstate_manipulation.so";

struct Application {
    library: Library,
}

impl Application {
    fn new() -> Self {
        let library = Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error));

        Application { library: library }
    }

    fn new_state(&self) -> State {
        unsafe {
            let f = self.library.get::<fn() -> State>(b"new_state\0").unwrap();
            f()
        }
    }

    fn update_and_render(&self,
                         platform: &Platform,
                         state: &mut State,
                         events: &Vec<Event>)
                         -> bool {
        unsafe {
            let f = self.library
                .get::<fn(&Platform, &mut State, &Vec<Event>) -> bool>(b"update_and_render\0")
                .unwrap();
            f(platform, state, events)
        }
    }
}

fn main() {
    terminal::open("____", 80, 30);
    terminal::set(config::Window::empty().resizeable(true));
    terminal::set(vec![config::InputFilter::Group {
                           group: config::InputFilterGroup::Keyboard,
                           both: false,
                       },
                       config::InputFilter::Group {
                           group: config::InputFilterGroup::Mouse,
                           both: false,
                       }]);



    let mut app = Application::new();

    let mut state = app.new_state();

    let mut last_modified = std::fs::metadata(LIB_PATH).unwrap().modified().unwrap();

    let platform = Platform {
        print_xy: terminal::print_xy,
        clear: clear,
        size: size,
        mouse_position: mouse_position,
        clicks: terminal::state::mouse::clicks,
        key_pressed: key_pressed,
        set_colors: set_colors,
    };

    let mut events = Vec::new();

    app.update_and_render(&platform, &mut state, &mut events);

    terminal::refresh();

    loop {
        events.clear();

        while let Some(event) = terminal::read_event() {
            events.push(event);
        }

        terminal::clear(None);

        if app.update_and_render(&platform, &mut state, &mut events) {
            //quit requested
            break;
        }

        terminal::refresh();

        if let Ok(Ok(modified)) = std::fs::metadata(LIB_PATH).map(|m| m.modified()) {
            if modified > last_modified {
                drop(app);
                app = Application::new();
                last_modified = modified;
            }
        }

    }

    terminal::close();
}

fn clear(area: Option<common::Rect>) {
    unsafe { terminal::clear(mem::transmute::<Option<common::Rect>, Option<Rect>>(area)) };
}

fn size() -> common::Size {
    unsafe { mem::transmute::<Size, common::Size>(state::size()) }
}

fn mouse_position() -> common::Point {
    unsafe { mem::transmute::<Point, common::Point>(state::mouse::position()) }
}

fn key_pressed(key: common::KeyCode) -> bool {
    terminal::state::key_pressed(unsafe { mem::transmute::<common::KeyCode, KeyCode>(key) })
}

fn set_colors(fg: common::Color, bg: common::Color) {
    terminal::set_colors(unsafe { mem::transmute::<common::Color, Color>(fg) },
                         unsafe { mem::transmute::<common::Color, Color>(bg) });

}
