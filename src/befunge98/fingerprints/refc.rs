use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(refc [r, d])
}

// Reference
fn refc_r(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let y = this.pop();
    let x = this.pop();
    this.push(state.refc_vectors.len() as i64);
    state.refc_vectors.push((x, y));
}

// Dereference
fn refc_d(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let id = this.pop();
    if let Some((x, y)) = state.refc_vectors.get(id as usize) {
        this.push(*x);
        this.push(*y);
    } else {
        this.direction = this.direction.reverse();
    }
}
