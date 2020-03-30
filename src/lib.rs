//! A macro for capturing variables on a per variable basis.
//!
//! With this macro it is possible to specifically designate which variables
//! will be captured by which method in a designated capture list.
//! Variables can be either specified to be moved, referenced, mutably
//! referenced or transformed using an arbitrary method specifier (e.g.,
//! `clone`).
//! Any variables not specifically designated will be moved by default.
//!
//! The specifiers for each capture type are:
//! - `move`
//! - `ref`
//! - `ref mut`
//! - $ident (any method identifier without arguments)
//!
//! This avoids having to manually declare references ahead of a move closure in
//! order to prevent unwanted moves.
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
//! let mutex = Arc::new(Mutex::new(Vec::new()));
//! let barrier = Arc::new(Barrier::new(2));
//!
//! let vector_clone = Arc::clone(&mutex);
//! let barrier_clone = Arc::clone(&barrier);
//!
//! thread::spawn(move || {
//!     let mut vec = vector_clone.lock().unwrap();
//!     vec.push(2);
//!     vec.push(3);
//!     vec.push(4);
//!
//!     barrier_clone.wait();
//! });
//!
//! barrier.wait();
//! let mut vec = mutex.lock().unwrap();
//!
//! vec.push(1);
//! assert_eq!(*vec, &[2, 3, 4, 1]);
//! ```
//!
//! You can now write:
//!
//! ```
//! use std::thread;
//! use std::sync::{Arc, Barrier, Mutex};
//!
//! use closure::closure;
//!
//! let mutex = Arc::new(Mutex::new(Vec::new()));
//! let barrier = Arc::new(Barrier::new(2));
//!
//! thread::spawn(closure!(clone mutex, clone barrier, || {
//!     let mut vec = mutex.lock().unwrap();
//!     vec.push(2);
//!     vec.push(3);
//!     vec.push(4);
//!
//!     barrier.wait();
//! }));
//!
//! barrier.wait();
//! let mut vec = mutex.lock().unwrap();
//!
//! vec.push(1);
//! assert_eq!(*vec, &[2, 3, 4, 1]);
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
//! let pair = Arc::new((Mutex::new(false), Condvar::new()));
//! let pair2 = pair.clone();
//!
//! // Inside of our lock, spawn a new thread, and then wait for it to start.
//! thread::spawn(move|| {
//!     let &(ref lock, ref cvar) = &*pair2;
//!     let mut started = lock.lock().unwrap();
//!     *started = true;
//!     // We notify the condvar that the value has changed.
//!     cvar.notify_one();
//! });
//!
//! // Wait for the thread to start up.
//! let &(ref lock, ref cvar) = &*pair;
//! let mut started = lock.lock().unwrap();
//! while !*started {
//!     started = cvar.wait(started).unwrap();
//! }
//! ```
//!
//! With `closure!`, the explicit declaration of `pair2` can be avoided:
//!
//! ```
//! use std::sync::{Arc, Mutex, Condvar};
//! use std::thread;
//!
//! use closure::closure;
//!
//! let pair = Arc::new((Mutex::new(false), Condvar::new()));
//!
//! // Inside of our lock, spawn a new thread, and then wait for it to start.
//! thread::spawn(closure!(clone pair, || {
//!     let &(ref lock, ref cvar) = &*pair;
//!     let mut started = lock.lock().unwrap();
//!     *started = true;
//!     // We notify the condvar that the value has changed.
//!     cvar.notify_one();
//! }));
//!
//! // Wait for the thread to start up.
//! let &(ref lock, ref cvar) = &*pair;
//! let mut started = lock.lock().unwrap();
//! while !*started {
//!     started = cvar.wait(started).unwrap();
//! }
//! ```
//!
//! ## Mixing move and reference captures without having to specifically declare
//! the references
//!
//! ```
//! use closure::closure;
//!
//! let move_string = String::from("This string should be moved");
//! let mut ref_string = String::from("This string will be referenced");
//!
//! let closure = closure!(move move_string, ref mut ref_string, || {
//!     ref_string.push_str(&move_string);
//!     //.. `move_string` is dropped at the end of the scope
//! });
//! ```
//!
//! Variable identifiers in the argument position (i.e., between the vertical
//! lines) and return type specifications can also be used same as in regular
//! closures.
//!
//! # Limitations
//!
//! Perhaps counter-intuitively, when designating a move variable, that variable
//! is only moved if it is actually used in the closure code.
//! Also, every closure given to the macro is invariably transformed to a move
//! closure, so `closure!(|| {...})` will move capture any variables in the
//! closure block.

//#![feature(trace_macros)]
//#![feature(log_syntax)]
//trace_macros!(true);

#[macro_export(local_inner_macros)]
macro_rules! closure_ext {
    (@inner move $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = $($ids).+;
        closure_ext!(@inner $($tail)*) 
    };
    (@inner ref $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = & $($ids).+;
        closure_ext!(@inner $($tail)*) 
    };
    (@inner ref mut $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = &mut $($ids).+;
        closure_ext!(@inner $($tail)*)
    };
    (@inner $fn:ident $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = $($ids).+.$fn();
        closure_ext!(@inner $($tail)*)
    };
    (@inner , $($tail:tt)*) => {
        closure_ext!(@inner $($tail)*)
    };
    // matches on the actual closure (w/o move)
    (@inner $($closure:tt)*) => {
        $crate::__assert_closure!($($closure)*);
        move $($closure)*
    };    
    // macro entry point (accepts anything)
    ($($args:tt)*) => {{
        closure_ext! { @inner $($args)* }
    }};
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __extract_last_ident {
    ($last:ident) => { $last };
    ($ignore:ident.$($tail:ident).+) => { $crate::__extract_last_ident!($($tail).+) };
}

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
    // Capture by IDENT (e.g., clone)
    (@inner $fn:ident $var:ident $($tail:tt)*) => {
        let $var = $var.$fn();
        closure!(@inner $($tail)*)
    };
    // Matches comma (at most one) between captures
    (@inner , $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    // Matches on the actual closure (with move, not permitted)
    (@inner move $($closure:tt)*) => {
        compile_error!("keyword `move` not permitted here.");
    };
    // Matches on the actual closure (w/o move)
    (@inner $($closure:tt)*) => {
        $crate::__assert_closure!($($closure)*);
        move $($closure)*
    };
    (, $($args:tt)*) => {
        compile_error!("closure capture list may not begin with a comma");
    };
    // Macro entry point (accepts anything)
    ($($args:tt)*) => {{
        closure!{@inner $($args)*}
    }};
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __assert_closure {
    (move $($any:tt)*) => { compile_error!("keyword `move` not permitted here") };
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
        bar: Bar,
    }

    struct Bar {
        baz: i32,
    }

    impl Foo {
        fn new(baz: i32) -> Self {
            Foo { bar: Bar { baz } }
        }
    }

    #[test]
    fn closure_ext() {
        let foo = Foo::new(0);
        let goo = 1;
        let arg = 2;
        let closure = closure_ext!(ref foo.bar.baz, ref goo, |arg: &i32| *baz == 0 && *goo == 1 && *arg == 2);
        assert!(closure(&arg));

        let closure = closure_ext!(|| true);
        assert!(closure());

        let var = "hello";
        let closure = closure_ext!(to_string var, || var == "hello");
        assert!(closure());
    }

    /*#[test]
    fn no_capture_one_line_1() {
        let closure = closure!(|| true);
        assert!(closure());
    }

    #[test]
    fn no_capture_one_line_2() {
        let closure = closure!(|| 5 * 5);
        assert_eq!(closure(), 25);
    }

    #[test]
    fn no_capture_with_arg() {
        let closure = closure!(|x| x * x);
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_arg_type_hint_1() {
        let closure = closure!(|x: usize| x * x);
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_arg_type_hint_2() {
        let closure = closure!(|x: usize| { x * x });
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_arg_and_return_type() {
        let closure = closure!(|x: usize| -> usize { x * x });
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_mut_arg() {
        let closure = closure!(|mut string: String, n: usize| -> usize {
            for _ in 0..n {
                string.push('x');
            }
            string.len()
        });

        assert_eq!(closure(String::from("xxxx"), 6), 10);
    }

    #[test]
    fn no_capture_w_return_type() {
        let closure = closure!(|| -> &str { "result" });
        assert_eq!(closure(), "result");
    }

    #[test]
    fn single_capture_move() {
        let string = String::from("move");
        let closure = closure!(move string, || string.len());
        assert_eq!(closure(), 4);
    }

    #[test]
    fn single_capture_move_mut() {
        let mut string = String::from("move");
        let closure = closure!(move string, || {
            string.clear();
            string.push_str("moved");
            string
        });

        assert_eq!(&closure(), "moved");
    }

    #[test]
    fn single_capture_ref() {
        let val = Foo::new(50);
        let closure = closure!(ref val, || {
            let inner = val.bar();
            assert_eq!(inner, 50);
        });

        closure();
    }

    #[test]
    fn single_capture_mut_ref() {
        let mut val = Foo::new(100);

        let mut closure = closure!(ref mut val, || {
            val.bar += 10;
        });
        closure();

        assert_eq!(val.bar, 110);
    }

    #[test]
    fn single_capture_clone() {
        use std::rc::Rc;

        let rc = Rc::new(50);
        let closure = closure!(clone rc, || -> usize {
            Rc::strong_count(&rc)
        });
        assert_eq!(2, closure());
    }

    #[test]
    fn single_capture_with_arg() {
        let mut val = Foo::new(10);
        let mut closure = closure!(ref mut val, |x: usize| {
            val.bar += x;
            val.bar
        });

        assert_eq!(closure(5), 15);
        assert_eq!(val.bar, 15);
    }

    #[test]
    fn multiple_capture() {
        let mut string = String::from("string");
        let x = 5;
        let mut y = 10;

        let mut closure = closure!(move string, ref x, ref mut y, || {
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
        let closure = closure!(move second, move third, |first| {
            format!("{} {} {}", first, second, third)
        });

        take_closure(closure);
    }

    #[test]
    fn arbitrary_fn_capture() {
        let str = "string";
        let closure = closure!(to_owned str, || {
            assert_eq!(String::from("string"), str.clone());
            str
        });

        let owned: String = closure();
        assert_eq!(&owned, str);
    }*/
}
