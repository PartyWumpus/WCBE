use std::time::Instant;

use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(hrti [g, m, t, e, s])
}

fn hrti_g(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    // timer granularity in microseconds
    this.push(1);
}

fn hrti_m(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    state.hrti_marks.insert(this.id, Instant::now());
}

fn hrti_t(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    if let Some(prev) = state.hrti_marks.get(&this.id) {
        let time_since = (Instant::now() - *prev).as_micros();
        this.push(time_since as i64);
    } else {
        this.direction = this.direction.reverse();
    };
}

fn hrti_e(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    state.hrti_marks.remove(&this.id);
}

fn hrti_s(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    this.push(state.hrti_start.elapsed().subsec_micros() as i64);
}
