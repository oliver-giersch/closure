//! A macro for capturing variables on a per variable basis.
//!
//! With this macro it is possible to specifically designate which variables will be captured by which
//! method.
//! Variables can be either specified to be moved, referenced, mutably referenced or cloned.
//! Unspecified variables will automatically be moved.
//!
//! The specifiers for each capture type are:
//! - move
//! - ref
//! - ref mut
//! - clone
//!
//! This avoids having to manually declare references ahead of a move closure in order to prevent
//! unwanted moves.
//!
//! # Examples
//!
//! ## Spawning a Thread
//!
//! Instead of having to write:
//!
//! ```
//! use std::thread;
//! use std::sync::{Arc, Barrier, Mutex};
//!
//! fn main() {
//!     let mutex = Arc::new(Mutex::new(Vec::new()));
//!     let barrier = Arc::new(Barrier::new(2));
//!
//!     let vector_clone = Arc::clone(&mutex);
//!     let barrier_clone = Arc::clone(&barrier);
//!
//!     thread::spawn(move || {
//!         let mut vec = vector_clone.lock().unwrap();
//!         vec.push(2);
//!         vec.push(3);
//!         vec.push(4);
//!
//!         barrier_clone.wait();
//!     });
//!
//!     barrier.wait();
//!     let mut vec = mutex.lock().unwrap();
//!
//!     vec.push(1);
//!     assert_eq!(*vec, &[2, 3, 4, 1]);
//! }
//! ```
//!
//! You can now write:
//!
//! ```
//! #[macro_use]
//! extern crate closure;
//!
//! use std::thread;
//! use std::sync::{Arc, Barrier, Mutex};
//!
//!
//! fn main() {
//!     let mutex = Arc::new(Mutex::new(Vec::new()));
//!     let barrier = Arc::new(Barrier::new(2));
//!
//!     thread::spawn(closure!(clone mutex, clone barrier || {
//!         let mut vec = mutex.lock().unwrap();
//!         vec.push(2);
//!         vec.push(3);
//!         vec.push(4);
//!
//!         barrier.wait();
//!     }));
//!
//!     barrier.wait();
//!     let mut vec = mutex.lock().unwrap();
//!
//!     vec.push(1);
//!     assert_eq!(*vec, &[2, 3, 4, 1]);
//! }
//! ```
//!
//! ## Mixing move and reference captures without having to specifically declare the references:
//!
//! ```
//! #[macro_use]
//! extern crate closure;
//!
//! use closure::*;
//!
//! fn main() {
//!     let move_string = String::from("This string should be moved");
//!     let mut ref_string = String::from("This string will be referenced");
//!
//!     let closure = closure!(move move_string, ref mut ref_string || {
//!         ref_string.push_str(&move_string);
//!         //move_string is dropped at the end of the scope
//!     });
//! }
//!
//! ```
//!
//! Variable identifiers in the argument position (between the vertical lines) can also be used same
//! as in regular closures.
//! Also, for the sake of completeness, the regular closure syntax can be used within the macro as
//! well.
//!
//! # Limitations
//!
//! Closure syntax specifying the argument and return types are currently not supported.

#[macro_export]
macro_rules! closure {
    (move || $f:expr) => {
        move || $f
    };
    (move |$($arg:ident $(: $t:ty)*),*| $f:expr) => {
        move |$($arg $(: $t)*),*| $f
    };
    (move |$($arg:ident $(: $t:ty)*),*| $f:expr) => {
        move |$($arg $(: $t)*),*| $f
    };
    (|| $f:expr) => {
        || $f
    };
    (|$($arg:ident),*| $f:expr) => {
        |$($arg),*| $f
    };
    (@inner || $f:expr) => {
        move || $f
    };
    (@inner |$($arg:ident),*| $f:expr) => {
        move |$($arg),*| $f
    };
    (@inner move $var:ident $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    (@inner ref mut $var:ident $($tail:tt)*) => {
        let $var = &mut $var;
        closure!(@inner $($tail)*)
    };
    (@inner ref $var:ident $($tail:tt)*) => {
        let $var = &$var;
        closure!(@inner $($tail)*)
    };
    (@inner clone $var:ident $($tail:tt)*) => {
        let $var = $var.clone();
        closure!(@inner $($tail)*)
    };
    (@inner , $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    ($($args:tt)*) => {{
        closure!{@inner $($args)*}
    }};
}

#[cfg(test)]
mod test {

    #[test]
    fn default_syntax() {
        let borrow = 5;
        let closure = closure!(|| assert_eq!(5, borrow));
        closure();

        let closure = closure!(|x| borrow + x);
        assert_eq!(25, closure(20));

        let string = String::from("move");
        let closure = closure!(move || assert_eq!("move", &string));
        closure();

        let string = String::from("move");
        let closure = closure!(move |x: usize| {
            string.len() + x
        });
        assert_eq!(5, closure(1));
    }

    #[test]
    fn no_capture() {
        let closure = closure!(|| true);
        assert_eq!(true, closure());

        let closure = closure!(|| 5 * 5);
        assert_eq!(25, closure());

        let closure = closure!(|| assert!(true));
        closure();
    }

    #[test]
    fn single_capture() {
        use std::rc::Rc;

        let mut string = String::from("initial");
        let closure = closure!(move string || {
            string.push_str(" (appended)");
            string
        });
        assert_eq!("initial (appended)", &closure());

        let x = 10;
        let closure = closure!(ref x || assert_eq!(10, *x));
        closure();

        let mut y = 5;

        let result = {
            let mut closure = closure!(ref mut y || {
                *y = *y * *y;
                *y - *y
            });
            closure()
        };

        assert_eq!(25, y);
        assert_eq!(0, result);

        let rc = Rc::new(50);
        let closure = closure!(clone rc || {
            assert_eq!(2, Rc::strong_count(&rc));
        });
        closure();
    }

    #[test]
    fn multiple_capture() {
        let mut string = String::from("string");
        let x = 5;
        let mut y = 10;

        let mut closure = closure!(move string, ref x, ref mut y || {
            string.push_str(" moved");
            assert_eq!("string moved", &string);
            assert_eq!(15, *x + *y);
        });
        closure();
    }

    #[test]
    fn multiple_capture_w_args() {
        #[inline]
        fn take_closure(closure: impl FnOnce(String) -> String) {
            let string = String::from("First");
            let result = closure(string);

            assert_eq!("First Second Third", &result);
        }

        let second = String::from("Second");
        let third = String::from("Third");
        let closure = closure!(move second, move third |first| {
            format!("{} {} {}", first, second, third)
        });

        take_closure(closure);
    }
}