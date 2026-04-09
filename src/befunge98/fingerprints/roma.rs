use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(roma [c, d, i, l, m, v, x])
}

fn roma_c(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(100)
}

fn roma_d(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(500)
}

fn roma_i(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(1)
}

fn roma_l(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(50)
}

fn roma_m(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(1000)
}

fn roma_v(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(5)
}

fn roma_x(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    this.push(10)
}
