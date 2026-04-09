use crate::{
    app::Settings,
    befunge98::{Cursor, Env},
};

pub type FingerprintFunction = fn(&mut Cursor, &mut Env, &Settings);
pub type Fingerprint = [Option<FingerprintFunction>; 26];

#[cfg(not(target_arch = "wasm32"))]
mod hrti;

mod bool;
mod null;
mod roma;

#[macro_export]
macro_rules! fingerprint_map {
    ($prefix:ident [$($enabled:ident),* $(,)?]) => {{
        let mut arr: Fingerprint = [None; 26];
        $(
            paste::paste! {
                arr[(stringify!($enabled).as_bytes()[0] - b'a') as usize] = Some([< $prefix _ $enabled >]);
            }
        )*
        arr
    }};
}

#[macro_export]
macro_rules! fingerprint_match {
    ($id:ident, [$($(#[$attr:meta])* $enabled:ident),* $(,)?]) => {{
        $(
            $(#[$attr])*
            paste::paste! {
                const [<$enabled:upper _ ID>]: i64 = fingerprint_to_i64(stringify!([<$enabled:upper>]).as_bytes());
            }
        )*

        match $id {
        $(
            $(#[$attr])*
            paste::paste! {
                [<$enabled:upper _ ID>]
            } => Some(paste::paste! { [<$enabled>]::list_of_ops() }),
        )*
        _ => None
        }
    }};
}

pub const fn fingerprint_to_i64(bytes: &[u8]) -> i64 {
    let mut num: i64 = 0;
    let mut i = 0;
    while i < bytes.len() {
        num *= 256;
        num += bytes[i] as i64;
        i += 1;
    }
    num
}

pub const fn fingerprint_from_id(id: i64) -> Option<Fingerprint> {
    fingerprint_match!(
        id,
        [
            null,
            roma,
            bool,
            #[cfg(not(target_arch = "wasm32"))]
            hrti
        ]
    )
}
