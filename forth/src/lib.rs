extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

mod forth;
use forth::{Forth, Error};

#[wasm_bindgen]
pub fn interpret(code: &str) -> String {
    let mut f = Forth::new();
    match f.eval(code) {
        Ok(()) => {
            let stack = f.stack();
            let stack_str = stack.into_iter().rev().map(|x| x.to_string()).collect::<Vec<_>>();
            let result = stack_str.connect("<br/>");
            return result;
        }
        Err(e) => {
            match e {
                Error::DivisionByZero => return String::from("Error: division by zero"),
                Error::StackUnderflow => return String::from("Error: stack underflow"),
                Error::UnknownWord => return String::from("Error: unknown word"),
                Error::InvalidWord => return String::from("Error: invalid word")
            }
        }
    }

}
