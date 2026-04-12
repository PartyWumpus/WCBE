use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(modu [m, u, r])
}

// signed-result modulo
fn modu_m(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    if a == 0 {
        this.push(0);
    } else {
        let modu = b - (b / a) * a;
        if (a < 0) == (b < 0) {
            this.push(modu);
        } else {
            this.push(-modu);
        }
    }
}

// Sam Holden's unsigned-result modulo
// hopefully... i don't understand what algorithm it's supposed to be
fn modu_u(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    if a == 0 {
        this.push(0)
    } else {
        this.push((b % a).abs())
    }
}

// C-language integer remainder
fn modu_r(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = this.pop();
    let b = this.pop();
    if a == 0 {
        this.push(0)
    } else {
        let modu = b % a;

        if (b < 0) == (modu < 0) {
            this.push(modu)
        } else {
            this.push(-modu)
        }
    };
}
