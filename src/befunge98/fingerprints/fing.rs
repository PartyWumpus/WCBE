use crate::{
    app::Settings,
    befunge98::{
        Cursor, Env,
        fingerprints::{Fingerprint, reflect_fn},
    },
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(fing [x, y, z])
}

fn pop_semantic(this: &mut Cursor) -> Option<usize> {
    let val = this.pop().try_into().ok()?;
    return match val {
        a @ 0..=25 => Some(a as usize),
        a @ b'A'..=b'Z' => Some((a - b'A') as usize),
        _ => None,
    };
}

fn fing_x(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let Some(a) = pop_semantic(this) else {
        this.direction = this.direction.reverse();
        return;
    };
    let Some(b) = pop_semantic(this) else {
        this.direction = this.direction.reverse();
        return;
    };
    let func_a = this.fingerprints[a].pop().unwrap_or(reflect_fn);
    let func_b = this.fingerprints[b].pop().unwrap_or(reflect_fn);
    this.fingerprints[a].push(func_b);
    this.fingerprints[b].push(func_a);
}

fn fing_y(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let Some(a) = pop_semantic(this) else {
        this.direction = this.direction.reverse();
        return;
    };
    this.fingerprints[a].pop();
}

fn fing_z(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let Some(a) = pop_semantic(this) else {
        this.direction = this.direction.reverse();
        return;
    };
    let Some(b) = pop_semantic(this) else {
        this.direction = this.direction.reverse();
        return;
    };
    let func = this.fingerprints[b].last().copied().unwrap_or(reflect_fn);
    this.fingerprints[a].push(func);
}
