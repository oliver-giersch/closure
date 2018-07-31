/*!
A macro for capturing variables on a per variable basis.

With this macro it is possible to specifically designate which variables will be captured by which
method.
Variables can be either specified to be moved, referenced, mutably referenced or cloned.
Unspecified variables will automatically be moved.

This avoids having to manually declare references ahead of a move closure in order to prevent
unwanted moves.

```ignore
let move_string = String::from("This string should be moved");
let mut ref_string = String::from("This string will be referenced);

let closure_ref = &mut ref_string;
let closure = move || {
    ref_string.push_str(&move_string));
};
```

# Syntax


```ìgnore
closure!(
    [move|ref|ref mut|clone] VARIABLE0, [move|ref|ref mut|clone] VARIABLE1, ... (optional)
    | ARG0, ARG1, ... (optional) |
    CLOSURE (expression or statement)
)
```

# Example

```rust

```

*/

#[macro_export]
macro_rules! closure {
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

fn main() {
    let x = 5;
    let mut y = 10;
    let z = 10;

    let result = {
        let mut closure = closure!(move x, ref mut y || {
            *y = 20;
            x + *y
        });
        closure()
    };

    assert_eq!(result, 25);
    assert_eq!(y, 20);
}

#[cfg(test)]
mod test {

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
        let mut closure = closure!(move string || {
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