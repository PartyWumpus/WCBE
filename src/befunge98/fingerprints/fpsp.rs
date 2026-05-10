use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(fpsp [a,b,c,d,e,f,g,h,i,k,l,m,n,p,q,r,s,t,v,x,y])
}

fn pop_float(this: &mut Cursor) -> f32 {
    f32::from_bits(this.pop() as u32)
}

fn push_float(this: &mut Cursor, val: f32) {
    this.push(val.to_bits() as i64)
}

fn fpsp_a(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    let b = pop_float(this);
    push_float(this, b + a);
}
fn fpsp_b(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.sin());
}
fn fpsp_c(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.cos());
}
fn fpsp_d(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    let b = pop_float(this);
    push_float(this, b / a);
}
fn fpsp_e(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.asin());
}
fn fpsp_f(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = this.pop();
    push_float(this, a as f32);
}
fn fpsp_g(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.atan());
}
fn fpsp_h(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.acos());
}
fn fpsp_i(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    this.push(a as i64);
}
fn fpsp_k(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.ln());
}
fn fpsp_l(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.log10());
}
fn fpsp_m(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    let b = pop_float(this);
    push_float(this, b * a);
}
fn fpsp_n(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, -a);
}
fn fpsp_p(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    state.output.push_str(&a.to_string());
}
fn fpsp_q(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.sqrt());
}
fn fpsp_r(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let Some(str) = this.pop_string() else {
        this.direction = this.direction.reverse();
        return;
    };

    if let Ok(f) = str.parse::<f32>() {
        push_float(this, f);
    } else {
        this.direction = this.direction.reverse();
    };
}
fn fpsp_s(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    let b = pop_float(this);
    push_float(this, b - a);
}
fn fpsp_t(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.tan());
}
fn fpsp_v(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.abs());
}
fn fpsp_x(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    push_float(this, a.exp());
}
fn fpsp_y(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_float(this);
    let b = pop_float(this);
    push_float(this, b.powf(a));
}
