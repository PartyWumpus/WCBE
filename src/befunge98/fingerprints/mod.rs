use crate::{
    app::Settings,
    befunge98::{Cursor, StateTempName},
};

#[cfg(not(target_arch = "wasm32"))]
mod hrti;

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

fn fingerprint_to_i64(str: &str) -> i64 {
    let mut num = 0;
    for char in str.chars() {
        num *= 256;
        num += char as i64;
    }
    num
}

pub type FingerprintFunction = fn(&mut Cursor, &mut StateTempName, &Settings);
pub type Fingerprint = [Option<FingerprintFunction>; 26];

pub const fn fingerprint_from_id(id: i64) -> Option<Fingerprint> {
    // TODO: macro this match too
    Some(match id {
        0x4e554c4c => null::list_of_ops(),
        0x524f4d41 => roma::list_of_ops(),

        #[cfg(not(target_arch = "wasm32"))]
        0x48525449 => hrti::list_of_ops(),
        _ => return None,
    })
}
