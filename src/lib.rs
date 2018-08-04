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
//! ## Moving cloned smart pointers into thread closures
//!
//! From the documentation of `std::sync::Condvar`:
//!
//! ```
//! use std::sync::{Arc, Mutex, Condvar};
//! use std::thread;
//!
//! fn main() {
//!     let pair = Arc::new((Mutex::new(false), Condvar::new()));
//!     let pair2 = pair.clone();
//!
//!     // Inside of our lock, spawn a new thread, and then wait for it to start.
//!     thread::spawn(move|| {
//!         let &(ref lock, ref cvar) = &*pair2;
//!         let mut started = lock.lock().unwrap();
//!         *started = true;
//!         // We notify the condvar that the value has changed.
//!         cvar.notify_one();
//!     });
//!
//!     // Wait for the thread to start up.
//!     let &(ref lock, ref cvar) = &*pair;
//!     let mut started = lock.lock().unwrap();
//!     while !*started {
//!         started = cvar.wait(started).unwrap();
//!     }
//! }
//! ```
//!
//! The declaration of `pair2` can be avoided:
//!
//! ```
//! #[macro_use]
//! extern crate closure;
//!
//! use std::sync::{Arc, Mutex, Condvar};
//! use std::thread;
//!
//! fn main() {
//!     let pair = Arc::new((Mutex::new(false), Condvar::new()));
//!
//!     // Inside of our lock, spawn a new thread, and then wait for it to start.
//!     thread::spawn(closure!(clone pair || {
//!         let &(ref lock, ref cvar) = &*pair;
//!         let mut started = lock.lock().unwrap();
//!         *started = true;
//!         // We notify the condvar that the value has changed.
//!         cvar.notify_one();
//!     }));
//!
//!     // Wait for the thread to start up.
//!     let &(ref lock, ref cvar) = &*pair;
//!     let mut started = lock.lock().unwrap();
//!     while !*started {
//!         started = cvar.wait(started).unwrap();
//!     }
//! }
//! ```
//!
//! ## Mixing move and reference captures without having to specifically declare the references
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
//! Variable identifiers in the argument position (between the vertical lines) and return type
//! specifications can also be used same as in regular closures.
//!
//! # Limitations
//!
//! Perhaps counter-intuitively, when designating a move variable, that variable is only moved if it is
//! actually used in the closure code.
//! Also, every closure given to the macro is invariably transformed to a move closure, so
//! `closure!(|| {...})` will move capture any variables in the closure block.

//#![feature(trace_macros)]
//#![feature(log_syntax)]
//trace_macros!(true);

#[macro_export]
macro_rules! closure {
    // Capture by move
    (@inner move $var:ident $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    // Capture by mutable reference
    (@inner ref mut $var:ident $($tail:tt)*) => {
        let $var = &mut $var;
        closure!(@inner $($tail)*)
    };
    // Capture by reference
    (@inner ref $var:ident $($tail:tt)*) => {
        let $var = &$var;
        closure!(@inner $($tail)*)
    };
    // Capture by cloning
    (@inner clone $var:ident $($tail:tt)*) => {
        let $var = $var.clone();
        closure!(@inner $($tail)*)
    };
    // Matches comma between captures
    (@inner , $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    // Matches on the actual closure (with move)
    (@inner move $($closure:tt)*) => {
        //__assert_closure!($($closure)*);
        //move $($closure)*
        compile_error!("keyword `move` not permitted here.");
    };
    // Matches on the actual closure (w/o move)
    (@inner $($closure:tt)*) => {
        __assert_closure!($($closure)*);
        move $($closure)*
    };
    // Macro entry point (accepts anything)
    ($($args:tt)*) => {{
        closure!{@inner $($args)*}
    }};
}

#[macro_export]
macro_rules! __assert_closure {
    (move $($any:tt)*) => {};
    (| $($any:tt)*) => {};
    (|| $($any:tt)*) => {};
    ($($any:tt)*) => {
        compile_error!(concat!(
            "the supplied argument is not a closure: `", stringify!($($any)*), "`")
            );
    };
}

#[cfg(test)]
mod test {
    struct Foo {
        bar : usize
    }

    impl Foo {
        fn new(bar: usize) -> Self {
            Foo { bar }
        }

        fn bar(&self) -> usize {
            self.bar
        }
    }

    #[test]
    fn no_capture_one_line_1() {
        let closure = closure!(|| true);
        assert_eq!(true, closure());
    }

    #[test]
    fn no_capture_one_line_2() {
        let closure = closure!(|| assert!(true));
        closure();
    }

    #[test]
    fn no_capture_one_line_3() {
        let closure = closure!(|| 5 * 5);
        assert_eq!(25, closure());
    }

    #[test]
    fn no_capture_with_arg() {
        let closure = closure!(|x| x * x);
        assert_eq!(25, closure(5));
    }

    #[test]
    fn no_capture_with_arg_type_hint_1() {
        let closure = closure!(|x: usize| x * x);
        assert_eq!(25, closure(5));
    }

    #[test]
    fn no_capture_with_arg_type_hint_2() {
        let closure = closure!(|x: usize| {
            x * x
        });
        assert_eq!(25, closure(5));
    }

    #[test]
    fn no_capture_with_arg_and_return_type() {
        let closure = closure!(|x: usize| -> usize {
            x * x
        });
        assert_eq!(25, closure(5));
    }

    #[test]
    fn no_capture_with_mut_arg() {
        let closure = closure!(|mut string: String, n: usize| -> usize {
            for _ in 0..n {
                string.push('x');
            }
            string.len()
        });

        let string = String::from("xxxx");
        assert_eq!(10, closure(string, 6));
    }

    #[test]
    fn no_capture_w_return_type() {
        let closure = closure!(|| -> &str {
            "result"
        });
        assert_eq!("result", closure());
    }

    #[test]
    fn single_capture_move() {
        let string = String::from("move");
        let closure = closure!(move string || string.len());
        assert_eq!(4, closure());
    }

    #[test]
    fn single_capture_move_mut() {
        let mut string = String::from("move");
        let closure = closure!(move string || {
            string.clear();
            string.push_str("moved");
            string
        });

        assert_eq!("moved", &closure());
    }

    #[test]
    fn single_capture_ref() {
        let foo = Foo::new(50);
        let closure = closure!(ref foo || {
            let bar = foo.bar();
            assert_eq!(50, bar);
        });

        closure();
    }

    #[test]
    fn single_capture_mut_ref() {
        let mut foo = Foo::new(100);

        {
            let mut closure = closure!(ref mut foo || {
                foo.bar += 10;
            });
            closure();
        }

        assert_eq!(110, foo.bar);
    }

    #[test]
    fn single_capture_clone() {
        use std::rc::Rc;

        let rc = Rc::new(50);
        let closure = closure!(clone rc || -> usize {
            Rc::strong_count(&rc)
        });
        assert_eq!(2, closure());
    }

    #[test]
    fn single_capture_with_arg() {
        let mut foo = Foo::new(10);
        let mut closure = closure!(ref mut foo |x: usize| {
            foo.bar += x;
            foo.bar
        });

        assert_eq!(15, closure(5));
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