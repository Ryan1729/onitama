
extern crate bear_lib_terminal;
extern crate common;

#[cfg(unix)]
extern crate libloading;
#[cfg(not(unix))]
extern crate state_manipulation;

#[cfg(unix)]
use libloading::Library;

use bear_lib_terminal::terminal::{self, config, Event, KeyCode, state};
use bear_lib_terminal::Color;
use bear_lib_terminal::geometry::{Point, Rect, Size};

use std::mem;

use common::*;

#[cfg(unix)]
const LIB_PATH: &'static str = "./target/debug/libstate_manipulation.so";
#[cfg(not(unix))]
const LIB_PATH: &'static str = "state_manipulation.dll";

#[cfg(unix)]
struct Application {
    library: Library,
}
#[cfg(not(unix))]
struct Application {}

#[cfg(unix)]
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
#[cfg(not(unix))]
impl Application {
    fn new() -> Self {
        Application {}
    }

    fn new_state(&self) -> State {
        state_manipulation::new_state()
    }

    fn update_and_render(&self,
                         platform: &Platform,
                         state: &mut State,
                         events: &Vec<Event>)
                         -> bool {
        let mut new_events: Vec<common::Event> =
            unsafe { events.iter().map(|a| mem::transmute::<Event, common::Event>(*a)).collect() };
        state_manipulation::update_and_render(platform, state, &mut new_events)
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
        pick: pick,
        mouse_position: mouse_position,
        clicks: terminal::state::mouse::clicks,
        key_pressed: key_pressed,
        set_colors: set_colors,
        get_colors: get_colors,
        set_layer: terminal::layer,
        get_layer: terminal::state::layer,
        set_foreground: set_foreground,
        get_foreground: get_foreground,
        set_background: set_background,
        get_background: get_background,
    };

    //if this isn't set to something explicitly `get_foreground`
    //will return 0 (transparent black) messing up code that
    //reads the foreground then sets a different one then sets
    // it back to what it was before.
    set_foreground(common::Color {
                       red: 255,
                       green: 255,
                       blue: 255,
                       alpha: 255,
                   });


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

//Note: index selects a cell in *a single* layer, in case you have composition mode on.
//To pick on different layers, set the current layer then pick.
fn pick(point: common::Point, index: i32) -> char {
    terminal::pick(unsafe { mem::transmute::<common::Point, Point>(point) },
                   index)
}

fn key_pressed(key: common::KeyCode) -> bool {
    terminal::state::key_pressed(unsafe { mem::transmute::<common::KeyCode, KeyCode>(key) })
}

fn set_colors(fg: common::Color, bg: common::Color) {
    terminal::set_colors(unsafe { mem::transmute::<common::Color, Color>(fg) },
                         unsafe { mem::transmute::<common::Color, Color>(bg) });

}

fn get_colors() -> (common::Color, common::Color) {
    (get_foreground(), get_background())
}

fn set_foreground(fg: common::Color) {
    terminal::set_foreground(unsafe { mem::transmute::<common::Color, Color>(fg) });
}
fn get_foreground() -> common::Color {
    unsafe { mem::transmute::<Color, common::Color>(terminal::state::foreground()) }
}
fn set_background(bg: common::Color) {
    terminal::set_background(unsafe { mem::transmute::<common::Color, Color>(bg) })
}
fn get_background() -> common::Color {
    unsafe { mem::transmute::<Color, common::Color>(terminal::state::background()) }
}
