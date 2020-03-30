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

#[macro_export(local_inner_macros)]
macro_rules! closure {
    (@inner move $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = $($ids).+;
        closure!(@inner $($tail)*)
    };
    (@inner move mut $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!(mut $($ids).+) = $($ids).+;
        closure!(@inner $($tail)*)
    };
    (@inner ref $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = & $($ids).+;
        closure!(@inner $($tail)*)
    };
    (@inner ref mut $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = &mut $($ids).+;
        closure!(@inner $($tail)*)
    };
    (@inner $fn:ident $($ids:ident).+ , $($tail:tt)*) => {
        let $crate::__extract_last_ident!($($ids).+) = $($ids).+.$fn();
        closure!(@inner $($tail)*)
    };
    (@inner , $($tail:tt)*) => {
        closure!(@inner $($tail)*)
    };
    // matches on the actual closure (w/o move)
    (@inner $($closure:tt)*) => {
        $crate::__assert_closure!($($closure)*);
        move $($closure)*
    };    
    // macro entry point (accepts anything)
    ($($args:tt)*) => {{
        closure! { @inner $($args)* }
    }};
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __extract_last_ident {
    ($last:ident) => { $last };
    (mut $last:ident) => { mut $last };
    ($ignore:ident.$($tail:ident).+) => { $crate::__extract_last_ident!($($tail).+) };
    (mut $ignore:ident.$($tail:ident).+) => { $crate::__extract_last_ident!(mut $($tail).+) };
}


#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __assert_closure {
    (| $($any:tt)*) => {};
    (|| $($any:tt)*) => {};
    (move $($any:tt)*) => { compile_error!("keyword `move` not permitted here") };
    ($($any:tt)*) => {
        compile_error!(concat!(
            "the supplied argument is not a closure: `", stringify!($($any)*), "`")
        );
    };
}

#[cfg(test)]
mod test {
    use crate::closure;

    struct Foo {
        bar: Bar,
    }

    #[derive(PartialEq, Eq)]
    struct Bar {
        baz: i32,
    }

    impl Foo {
        fn new(baz: i32) -> Self {
            Foo { bar: Bar { baz } }
        }

        fn consume(self) -> Box<dyn Fn(i32) -> bool> {
            Box::new(closure!(move self.bar.baz, |expected| baz == expected))
        }

        fn borrow(&self) -> Box<dyn Fn(i32) -> bool + '_> {
            Box::new(closure!(ref self.bar.baz, |expected| *baz == expected))
        }
    }

    #[test]
    fn no_capture_one_line() {
        let closure = closure!(|| 5 * 5);
        assert_eq!(closure(), 25);
    }

    #[test]
    fn no_capture_with_arg() {
        let closure = closure!(|x| x * x);
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_arg_and_type_hint() {
        let closure = closure!(|x: usize| x * x);
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_arg_and_return_type() {
        let closure = closure!(|x: usize| -> usize { x * x });
        assert_eq!(closure(5), 25);
    }

    #[test]
    fn no_capture_with_return_type() {
        let closure = closure!(|| -> &str { "result" });
        assert_eq!(closure(), "result");
    }

    #[test]
    fn capture_by_move() {
        let string = "move".to_string();
        let closure = closure!(move string, || string.len());
        assert_eq!(closure(), 4);
    }

    #[test]
    fn capture_by_ref() {
        let var = -1;
        let closure = closure!(ref var, || *var == -1);
        assert!(closure());
    }

    #[test]
    fn capture_by_ref_mut() {
        let mut var = -1;
        closure!(ref mut var, || *var *= -1)();
        assert_eq!(var, 1);
    }

    #[test]
    fn capture_nested_by_move() {
        let foo = Foo::new(-1);
        let closure = closure!(move foo.bar, || bar == Bar { baz: -1 });
        assert!(closure());
    }

    #[test]
    fn capture_nested_by_ref() {
        let foo = Foo::new(-1);
        let closure = closure!(ref foo.bar, || *bar == Bar { baz: -1 });
        assert!(closure());
    }

    #[test]
    fn capture_nested_by_ref_mut() {
        let mut foo = Foo::new(-1);
        closure!(ref mut foo.bar.baz, |add| *baz += add)(2);
        assert_eq!(foo.bar.baz, 1);
    }

    #[test]
    fn capture_nested_with_self_by_move() {
        let foo = Foo::new(-1);
        let closure = foo.consume();
        assert!(closure(-1));
    }

    #[test]
    fn capture_nested_with_self_by_ref() {
        let foo = Foo::new(-1);
        let closure = foo.borrow();
        assert!(closure(-1));
    }

    #[test]
    fn capture_multiple_mixed() {
        let borrow = 1;
        let mut borrow_mut = 1;
        let string = "move".to_string();

        let closure = closure!(ref borrow, ref mut borrow_mut, move mut string, || {
            assert_eq!(*borrow, 1);
            *borrow_mut -= 1;
            string.push_str("d back");
            string
        });

        assert_eq!(&closure(), "moved back");
    }

    #[test]
    fn capture_by_clone() {
        use std::rc::Rc;

        let rc = Rc::new(Foo::new(0));
        let closure = closure!(clone rc, |expected| -> bool {
            rc.bar.baz == expected && Rc::strong_count(&rc) == 2
        });
        assert!(closure(0));
    }

    #[test]
    fn capture_by_fn_ident() {
        let string = "string";
        let closure = closure!(to_string string, || {
            let mut owned: String = string;
            owned.push_str(", now owned");
            owned
        });

        assert_eq!(closure(), "string, now owned");
    }
}
