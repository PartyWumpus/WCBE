use egui::Color32;

use crate::{
    app::Settings,
    befunge::{GraphicalEvent, Graphics},
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(pixl [c, f, l, s, u, x, z])
}

// TODO: consider moving the current color to per-cursor so fun forking stuff can happen

// If i was actually using SDL2 like befunge-with-graphics then obviously "SDL2" would be a great
// fingerprint name, but I'm not so PIXL (pixel graphics) it is.

// Fill screen with color
fn pixl_c(_this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(graphics) = &mut state.graphics {
        graphics.fill();
    }
}

// Set color
fn pixl_f(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(graphics) = &mut state.graphics {
        let r = this.pop().try_into();
        let g = this.pop().try_into();
        let b = this.pop().try_into();
        if let Ok(r) = r
            && let Ok(g) = g
            && let Ok(b) = b
        {
            graphics.current_color = Color32::from_rgb(r, g, b);
        } else {
            //return StepStatus::Error("Out of bounds graphical operation");
            this.direction = this.direction.reverse();
        }
    }
}

// Draw line
fn pixl_l(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(graphics) = &mut state.graphics {
        let y1: i32 = this.pop().try_into().unwrap();
        let x1: i32 = this.pop().try_into().unwrap();

        let y2: i32 = this.pop().try_into().unwrap();
        let x2: i32 = this.pop().try_into().unwrap();

        if x1 >= graphics.size.0 as i32
            || y1 >= graphics.size.1 as i32
            || x2 >= graphics.size.0 as i32
            || y2 >= graphics.size.1 as i32
        {
            //return StepStatus::Error("Out of bounds graphical operation");
            this.direction = this.direction.reverse();
            return;
        }

        graphics.line(x1, y1, x2, y2);
    }
}

// Create screen
fn pixl_s(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let y = this.pop();
    let x = this.pop();

    if y <= 0 || x <= 0 || x > Graphics::MAX_IMAGE_SIZE || y > Graphics::MAX_IMAGE_SIZE {
        //return StepStatus::Error("Out of bounds graphical operation");
        this.direction = this.direction.reverse();
        return;
    }

    state.graphics = Some(Graphics::new(x as usize, y as usize));
}

fn pixl_u(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    // FIXME: should do a frame pause. need to get these functions to return step status
}

// Draw a pixel
fn pixl_x(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(graphics) = &mut state.graphics {
        let y = this.pop();
        let x = this.pop();

        //return graphics.pixel(x, y);
        graphics.pixel(x, y);
    }
}

// Poll an event
fn pixl_z(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(graphics) = &mut state.graphics {
        if let Some(event) = graphics.event_queue.pop_front() {
            match event {
                //None is event 0
                GraphicalEvent::Close => this.toss().extend([1]),
                //Event::KeyDown(key) => this.stack.extend([key,2]),
                //Event::KeyUp(key) => this.stack.extend([key,3]),
                GraphicalEvent::MouseClick((x, y)) => this.toss().extend([x, y, 4]),
            }
        } else {
            this.toss().push(0);
        }
    }
}
