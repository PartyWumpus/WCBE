use crate::{
    app::Settings,
    befunge98::{Cursor, StateTempName, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(bool [a, n, o, x])
}

fn bool_a(this: &mut Cursor, _state: &mut StateTempName, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    this.push(a & b);
}
fn bool_n(this: &mut Cursor, _state: &mut StateTempName, _settings: &Settings) {
    let a = this.pop();
    this.push(!a);
}
fn bool_o(this: &mut Cursor, _state: &mut StateTempName, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    this.push(a | b);
}
fn bool_x(this: &mut Cursor, _state: &mut StateTempName, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    this.push(a ^ b);
}
