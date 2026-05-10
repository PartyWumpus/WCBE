use crate::{
    app::Settings,
    befunge98::{Cursor, Env, fingerprints::Fingerprint},
    fingerprint_map,
};

pub const fn list_of_ops() -> Fingerprint {
    fingerprint_map!(fpdp [a,b,c,d,e,f,g,h,i,k,l,m,n,p,q,r,s,t,v,x,y])
}

fn pop_double(this: &mut Cursor) -> f64 {
    f64::from_bits(this.pop() as u64)
}

fn push_double(this: &mut Cursor, val: f64) {
    this.push(val.to_bits() as i64)
}

fn fpdp_a(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    let b = pop_double(this);
    push_double(this, b + a);
}
fn fpdp_b(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.sin());
}
fn fpdp_c(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.cos());
}
fn fpdp_d(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    let b = pop_double(this);
    push_double(this, b / a);
}
fn fpdp_e(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.asin());
}
fn fpdp_f(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = this.pop();
    push_double(this, a as f64);
}
fn fpdp_g(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.atan());
}
fn fpdp_h(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.acos());
}
fn fpdp_i(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    this.push(a as i64);
}
fn fpdp_k(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.ln());
}
fn fpdp_l(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.log10());
}
fn fpdp_m(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    let b = pop_double(this);
    push_double(this, b * a);
}
fn fpdp_n(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, -a);
}
fn fpdp_p(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    state.output.push_str(&a.to_string());
}
fn fpdp_q(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.sqrt());
}
fn fpdp_r(this: &mut Cursor, state: &mut Env, _settings: &Settings) {
    let Some(str) = this.pop_string() else {
        this.direction = this.direction.reverse();
        return;
    };

    if let Ok(f) = str.parse::<f64>() {
        push_double(this, f);
    } else {
        this.direction = this.direction.reverse();
    };
}
fn fpdp_s(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    let b = pop_double(this);
    push_double(this, b - a);
}
fn fpdp_t(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.tan());
}
fn fpdp_v(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.abs());
}
fn fpdp_x(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    push_double(this, a.exp());
}
fn fpdp_y(this: &mut Cursor, _state: &mut Env, _settings: &Settings) {
    let a = pop_double(this);
    let b = pop_double(this);
    push_double(this, b.powf(a));
}
