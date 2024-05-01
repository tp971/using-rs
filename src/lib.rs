//! This crate provides the [`using`] macro designed to simplify writing and using builders by
//! providing [method cascading](https://en.wikipedia.org/wiki/Method_cascading):
//!
//! ```
//! # use using::using;
//! #[derive(Debug, Copy, Clone)]
//! struct Vec3 {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! #[derive(Default, Debug, Copy, Clone)]
//! struct Vec3Builder {
//!     x: Option<f32>,
//!     y: Option<f32>,
//!     z: Option<f32>,
//! }
//!
//! impl Vec3Builder {
//!     pub fn x(&mut self, x: f32) {
//!         self.x = Some(x);
//!     }
//!
//!     pub fn y(&mut self, y: f32) {
//!         self.y = Some(y);
//!     }
//!
//!     pub fn z(&mut self, z: f32) {
//!         self.z = Some(z);
//!     }
//!
//!     //this also works with `self` instead of `&mut self`
//!     pub fn build(&mut self) -> Vec3 {
//!         Vec3 {
//!             x: self.x.unwrap(),
//!             y: self.y.unwrap(),
//!             z: self.z.unwrap(),
//!         }
//!     }
//! }
//!
//! let vec3 = using!(Vec3Builder::default() => {
//!     .x(4.27);
//!     .y(9.71);
//!     .z(13.37);
//!     .build()
//! });
//!
//! // Generated code:
//! //
//! // let vec3 = {
//! //     let mut target = Vec3Builder::default();
//! //     target.x(4.27);
//! //     target.y(9.71);
//! //     target.z(13.37);
//! //     target.build()
//! // };
//! ```
//!
//! The idea is that instead of implementing builders as fluid interfaces that allow method
//! chaining (i.e. each method returning `&mut Self` or `Self`), we implement our builder with
//! simple setter methods and use it with the [`using`] macro, which gives us the ergonomics of
//! conventional builders without having to implement the builder as a fluid interface.
//!
//! The macro is not bound to builders: it has no requirements on the type, therefore we can use it
//! on basically anything:
//!
//! ```
//! # use std::collections::HashMap;
//! # use using::using;
//! let map = using!(HashMap::new() => {
//!     .insert("a", 41);
//!     .insert("b", 971);
//! });
//! ```
//!
//! ```
//! # use using::using;
//! let hello_world = using!(Vec::new() => {
//!     .push("Hello");
//!     .push("World!");
//!     .join(", ")
//! });
//! assert_eq!(hello_world, "Hello, World!");
//! ```
//!
//! # Motivation
//!
//! The idea for this crate came from implementing the builder pattern in a personal project. In
//! Rust, there are three main approaches for designing builder structs:
//!
//! * All methods taking `self` and returning `Self`:
//!
//!   ```ignore
//!   impl SomeBuilder {
//!       pub fn new() -> Self { ... }
//!       pub fn x(self, arg: T) -> Self { ... }
//!       pub fn y(self, arg: U) -> Self { ... }
//!       pub fn z(self, arg: V) -> Self { ... }
//!       pub fn build(self) -> Something { ... }
//!   }
//!   ```
//!
//!   The advantage of this method is that when building the final object, the fields can be moved
//!   out of the builder. One disadvantage of this method is that using the builder in more
//!   complicated ways can become quite verbose: if a method must be called inside an `if`
//!   statement or a loop or if the builder must be passed to a function, the builder has to be
//!   stored in a mutable variable and re-assigned everytime:
//!
//!   ```ignore
//!   let mut builder = SomeBuilder::new()
//!       .x(...)
//!       .y(...);
//!   if some_condition {
//!       builder = builder.z(...);
//!   }
//!   if some_other_condition {
//!       builder = some_function(builder);
//!   }
//!   let thing = builder.build();
//!   ```
//!
//!   Also, the builder methods are quite verbose since they have to return `self`.
//!
//! * All methods taking `&mut self` and returning `&mut Self`:
//!
//!   ```ignore
//!   impl SomeBuilder {
//!       pub fn new() -> Self { ... }
//!       pub fn x(&mut self, arg: T) -> &mut Self { ... }
//!       pub fn y(&mut self, arg: U) -> &mut Self { ... }
//!       pub fn z(&mut self, arg: V) -> &mut Self { ... }
//!       pub fn build(&mut self) -> Something { ... }
//!   }
//!   ```
//!
//!   This improves the disadvantage of the first method with respect to more complicated
//!   use-cases, because there are no re-assignments:
//!
//!   ```ignore
//!   let mut builder = SomeBuilder::new()
//!       .x(...)
//!       .y(...);
//!   if some_condition {
//!       builder.z(...);
//!   }
//!   if some_other_condition {
//!       some_function(&mut builder);
//!   }
//!   let thing = builder.build();
//!   ```
//!
//!   However, with this method, the `build` method cannot take `self`, otherwise method chaining
//!   does not work (except we require a call to `clone` or something similar, which is not really
//!   intuitive). Therefore, we cannot just move out of `self`, so we might end up in situations
//!   where we have to clone objects to be put into the final objects or we have to move out of the
//!   builder and leave the builder in a state where calling `build` again would have a different
//!   behavior, which, again, is unintuitive.
//!
//! * Combining the two approaches above, e.g. by implementing methods `xyz` and `with_xyz`, where
//!   `xyz` takes `&mut self` and `with_xyz` takes `self`. This combines the advantages of both
//!   methods, but it makes defining the builder even more verbose and also requires at least one
//!   of the two methods for each field to have a longer name.
//!
//! A problem shared amongst all the approaches above is that having conditionals or loops around
//! calls to the builder break method chaining.
//!
//! The idea of this crate comes from the observation that the main reason builders are usually
//! designed as fluid interfaces is that we want to express the pattern "here is an object and I
//! want to call these methods on it" without explicitly defining the variable or referencing it
//! everytime. Therefore, we introduce a hypothetical language construct that does exactly that:
//!
//! ```ignore
//! let thing = using builder @ SomeBuilder::new() {
//!     x(...);
//!     y(...);
//!     if some_condition {
//!         z(...);
//!     }
//!     if some_other_condition {
//!         some_function(&mut builder);
//!     }
//!     build()
//! };
//! ```
//!
//! This hypothetical `using` expression takes an expression of any type (with an optional
//! @-binding) and a block expression. Inside that block, every public method and every public
//! field of that type is in the local scope of that block. With that, the example above would be
//! equivalent to:
//!
//! ```ignore
//! let thing = {
//!     let mut builder = SomeBuilder::new();
//!     builder.x(...);
//!     builder.y(...);
//!     if some_condition {
//!         builder.z(...);
//!     }
//!     if some_other_condition {
//!         some_function(&mut builder);
//!     }
//!     builder.build()
//! };
//! ```
//!
//! This is also known as [Method cascading](https://en.wikipedia.org/wiki/Method_cascading) and is
//! an actual feature in some languages, notably Pascal and Visual Basic (initiated with the
//! keyword `with`; we only chose `using` because the crate name was free ¯\\\_(ツ)\_/¯).
//!
//! The [`using`] macro emulates this behavior, with some restrictions due to the way macros are
//! interpreted, e.g. in the context of macros, we do not know the type of the given expression and
//! its public symbols, therefore we have to prefix method calls with a dot. Also, this way of
//! accessing members does not work in all contexts; for details, see the documentation of
//! [`using`].
//!
//! Writing builders with the [`using`] macro can be done by just defining a simple setter method
//! for each field, making the code for builder very concise. If the to-be-constructed struct is
//! simple enough, this could even make defining a builder obsolete. Also, the `build` method can
//! now take both `self` or `&mut self` without breaking method chaining, which is usually a
//! drawback of defining builders taking `&mut self`.

#![cfg_attr(not(test), no_std)]

/// A macro that provides method cascading for an object.
///
/// # Usage
///
/// ```plain
/// using!(expression => { ... })
///
/// using!(identifier @ expression => { ... })
/// ```
///
/// Binds `expression` to a mutable variable (called "target") that can be manipulated inside the
/// block with expressions starting with a dot (called "target expressions"). The target variable
/// can be explicitly named with an @-binding. If the block does not contain a trailing expression,
/// the target is returned instead.
///
/// Target expression are a sequence of field accessess (e.g. `.x`) and method calls (e.g.
/// `.push(10)`) and can only be used in blocks, let statements, bodies of if expressions, match
/// expressions, and loops. They cannot be used in the conditional expressions and also not in
/// compound expressions, e.g. `.last().unwrap() + 1` is not valid. For details see below.
///
/// Besides the target expressions, every statement and expression can be used inside the block,
/// which also allows nesting [`using`] macros.
///
/// # Examples:
///
/// ```
/// # use using::using;
/// let hello_world = using!(Vec::new() => {
///     .push("Hello");
///     .push("World!");
///     .join(", ")
/// });
/// assert_eq!(hello_world, "Hello, World!");
///
/// // Generated code:
/// //
/// // let hello_world = {
/// //     let mut target = Vec::new();
/// //     target.push("Hello");
/// //     target.push("World!");
/// //     target.join(", ")
/// // };
/// ```
///
/// More complicated example with `for`, `if`, and `let`:
///
/// ```
/// # use using::using;
/// let vec = using!(Vec::new() => {
///     for i in 0..10 {
///         if i % 2 == 0 {
///             .push(i);
///         }
///     }
///     let sum = .iter().sum();
///     .push(sum);
/// });
/// assert_eq!(&vec[..], [ 0, 2, 4, 6, 8, 20 ]);
/// ```
///
/// # Syntax:
///
/// This section explains the syntax in a BNF-like form to clarify the details and where target
/// expressions can be used. The symbols `IDENTIFIER`, `Statement`, `Expression`,
/// `BlockExpression`, `Pattern`, `GenericArgs`, `CallParams`, and `Type` are defined in [The Rust
/// Reference](https://doc.rust-lang.org/stable/reference/). The syntax of the macro is defined by:
///
/// ```plain
/// "using" "!" "(" Expression "=>" UsingBlock ")"
///
/// "using" "!" "(" IDENTIFIER "@" Expression "=>" UsingBlock ")"
/// ```
///
/// A `UsingBlock` is an extension of Rusts `BlockExpression`: it is a block surrounded by curly
/// braces, containing a sequence of `UsingStatement`s followed by an optional `UsingExpression`.
///
/// A `UsingStatement` is either a `Statement` or one of the following:
///
/// ```plain
/// UsingExpression ";"
///
/// "let" IDENTIFIER ( ":" Type )? = UsingExpression ";"
/// ```
///
/// A `UsingExpression` is either an `Expression` or one of the following:
///
/// ```plain
/// UsingBlock
///
/// // This defines the "target expressions"
/// ( "." IDENTIFIER | "." IDENTIFIER ( "::" GenericArgs )? "(" CallParams? ")" )+
///
/// "if" Expression UsingBlock ( "else" "if" Expression UsingBlock )* ( "else" UsingBlock )?
///
/// "match" Expression "{" ( Pattern ( "if" Expression )? => ( UsingBlock | UsingExpression "," ) )* "}"
///
/// "loop" UsingBlock
///
/// "while" Pattern "in" Expression UsingBlock
///
/// "for" Pattern "in" Expression UsingBlock
/// ```
#[macro_export]
macro_rules! using {
    ($target:expr => { $( $t:tt )* }) => {
        {
            #[allow(unused_mut)]
            let mut target = $target;
            $crate::using_impl!(target root empty { $($t)* })
        }
    };
    ($id:ident @ $target:expr => { $( $t:tt )* }) => {
        {
            #[allow(unused_mut)]
            let mut $id = $target;
            $crate::using_impl!($id root empty { $($t)* })
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! using_impl {
    ($target:ident $scope:ident maybe_trailing_exp ($id:ident) { }) => {
        $id
    };

    ($target:ident $scope:ident maybe_trailing_exp ($id:ident) { ; $($rest:tt)* }) => {
        $crate::using_impl!($target $scope empty { $($rest)* })
    };

    ($target:ident $scope:ident maybe_trailing_exp ($id:ident) { $($rest:tt)* }) => {
        $crate::using_impl!($target $scope empty { $($rest)* })
    };



    ($target:ident root empty { }) => {
        $target
    };

    ($target:ident block empty { }) => {
        #[allow(unreachable_code)]
        ()
    };

    ($target:ident $scope:ident empty { ; $($rest:tt)* }) => {
        {
            ;
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { . $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_exp ($target) { . $($rest)* })
    };

    ($target:ident $scope:ident in_exp ($exp:expr) { . $name:ident $( ::<$($ty:ty),* $(,)?> )? ( $($args:expr),* $(,)? ) $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_exp ($exp.$name$(::<$($ty),*>)*($($args),*)) { $($rest)* })
    };

    ($target:ident $scope:ident in_exp ($exp:expr) { . $name:ident $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_exp ($exp.$name) { $($rest)* })
    };

    ($target:ident $scope:ident in_exp ($exp:expr) { }) => {
        $exp
    };

    ($target:ident $scope:ident in_exp ($exp:expr) { ; $($rest:tt)* }) => {
        {
            $exp;
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };

    ($target:ident $scope:ident in_exp ($exp:expr) { . $name:ident = $value:expr; $($rest:tt)* }) => {
        {
            $exp.$name = $value;
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { { $($block:tt)* } }) => {
        $crate::using_impl!($target block empty { $($block)* })
    };

    ($target:ident $scope:ident empty { { $($block:tt)* } $($rest:tt)* }) => {
        {
            $crate::using_impl!($target block empty { $($block)* });
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { let $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_let () { $($rest)* })
    };

    ($target:ident $scope:ident in_let
        ($($pattern:tt)*)
        { = $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_let_exp ($($pattern)*) (_) () { $($rest)* })
    };

    ($target:ident $scope:ident in_let
        ($($pattern:tt)*)
        { : $ty:ty = $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_let_exp ($($pattern)*) ($ty) () { $($rest)* })
    };

    ($target:ident $scope:ident in_let
        ($($pattern:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_let ($($pattern)* $t) { $($rest)* })
    };

    ($target:ident $scope:ident in_let_exp
        ($pattern:pat)
        ($ty:ty)
        ($($exp:tt)*)
        { ; $($rest:tt)* }
    ) => {
        {
            let $pattern: $ty = $crate::using_impl!($target block empty { $($exp)* });
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };

    ($target:ident $scope:ident in_let_exp
        ($pattern:pat)
        ($ty:ty)
        ($($exp:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_let_exp ($pattern) ($ty) ($($exp)* $t) { $($rest)* })
    };



    ($target:ident $scope:ident empty { if $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_if () () () { $($rest)* })
    };

    ($target:ident $scope:ident in_if
        ($($if_curr:tt)*)
        ()
        ()
        { { $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_if_next
            ()
            (($($if_curr)*) { $($body)* })
            ()
            { $($rest)* }
        )
    };

    ($target:ident $scope:ident in_if
        ($($if_curr:tt)*)
        ($($if_first:tt)*)
        ($($if_rest:tt)*)
        { { $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_if_next
            ()
            ($($if_first)*)
            ($($if_rest)* (($($if_curr)*) { $($body)* }))
            { $($rest)* }
        )
    };

    ($target:ident $scope:ident in_if
        ($($if_curr:tt)*)
        ($($if_first:tt)*)
        ($($if_rest:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_if
            ($($if_curr)* $t)
            ($($if_first)*)
            ($($if_rest)*)
            { $($rest)* }
        )
    };

    ($target:ident $scope:ident in_if_next
        ()
        ($($if_first:tt)*)
        ($($if_rest:tt)*)
        { else if $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_if
            ()
            ($($if_first)*)
            ($($if_rest)*)
            { $($rest)* }
        )
    };

    ($target:ident $scope:ident in_if_next
        ()
        (($($if_first_cond:tt)*) { $($if_first_body:tt)* })
        ($( (($($if_rest_cond:tt)*) { $($if_rest_body:tt)* }) )*)
        { else { $($body:tt)* } $($rest:tt)* }
    ) => {
        {
            let _tmp = if $($if_first_cond)* {
                $crate::using_impl!($target block empty { $($if_first_body)* })
            } $( else if $($if_rest_cond)* {
                $crate::using_impl!($target block empty { $($if_rest_body)* })
            } )* else {
                $crate::using_impl!($target block empty { $($body)* })
            };
            $crate::using_impl!($target $scope maybe_trailing_exp (_tmp) { $($rest)* })
        }
    };

    ($target:ident $scope:ident in_if_next
        ()
        (($($if_first_cond:tt)*) { $($if_first_body:tt)* })
        ($( (($($if_rest_cond:tt)*) { $($if_rest_body:tt)* }) )*)
        { $($rest:tt)* }
    ) => {
        {
            if $($if_first_cond)* {
                $crate::using_impl!($target block empty { $($if_first_body)* })
            } $( else if $($if_rest_cond)* {
                $crate::using_impl!($target block empty { $($if_rest_body)* })
            } )*
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { match $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_match () { $($rest)* })
    };

    ($target:ident $scope:ident in_match
        ($($match_cond:tt)*)
        { { $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body ($($match_cond)*) () { { $($body)* } $($rest)* })
    };

    ($target:ident $scope:ident in_match
        ($($match_cond:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match ($($match_cond)* $t) { $($rest)* })
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        { { $pattern:pat $( if $guard:expr )? => . $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body_in_exp
            ($($match_cond)*)
            ($($match_cases)*)
            (($pattern) $($guard)*)
            (.)
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body_in_exp
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        (($match_pattern:pat) $($match_guard:expr)?)
        ($($match_exp:tt)*)
        { { , $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($match_pattern $( if $match_guard )* => { $($match_exp)* }))
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body_in_exp
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        (($match_pattern:pat) $($match_guard:expr)?)
        ($($match_exp:tt)*)
        { { } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($match_pattern $( if $match_guard )* => { $($match_exp)* }))
            { { } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body_in_exp
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        (($match_pattern:pat) $($match_guard:expr)?)
        ($($match_exp:tt)*)
        { { $t:tt $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body_in_exp
            ($($match_cond)*)
            ($($match_cases)*)
            (($match_pattern) $($match_guard)*)
            ($($match_exp)* $t)
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        { { $pattern:pat $( if $guard:expr )? => { $($exp:tt)* }, $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($pattern $( if $guard )* => { $($exp)* }))
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        { { $pattern:pat $( if $guard:expr )? => { $($exp:tt)* } $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($pattern $( if $guard )* => { $($exp)* }))
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        { { $pattern:pat $( if $guard:expr )? => $exp:expr, $($body:tt)* } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($pattern $( if $guard )* => { $exp }))
            { { $($body)* } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($($match_cases:tt)*)
        { { $pattern:pat $( if $guard:expr )? => $exp:expr } $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_match_body
            ($($match_cond)*)
            ($($match_cases)* ($pattern $( if $guard )* => { $exp }))
            { { } $($rest)* }
        )
    };

    ($target:ident $scope:ident in_match_body
        ($($match_cond:tt)*)
        ($( ($pattern:pat $( if $guard:expr )? => { $($exp:tt)* }) )*)
        { { } $($rest:tt)* }
    ) => {
        {
            let _tmp = match $($match_cond)* {
                $( $pattern $( if $guard )* => { $crate::using_impl!($target block empty { $($exp)* }) }, )*
            };
            $crate::using_impl!($target $scope maybe_trailing_exp (_tmp) { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { loop { $($body:tt)* } $($rest:tt)* }) => {
        {
            let _tmp = loop {
                $crate::using_impl!($target block empty { $($body)* })
            };
            $crate::using_impl!($target $scope maybe_trailing_exp (_tmp) { $($rest)* })
        }
    };



    ($target:ident $scope:ident empty { while $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_while () { $($rest)* })
    };

    ($target:ident $scope:ident in_while
        ($($while_cond:tt)*)
        { { $($body:tt)* } $($rest:tt)* }
    ) => {
        {
            while $($while_cond)* {
                $crate::using_impl!($target block empty { $($body)* })
            }
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };

    ($target:ident $scope:ident in_while
        ($($while_cond:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_while ($($while_cond)* $t) { $($rest)* })
    };



    ($target:ident $scope:ident empty { for $for_pattern:pat in $($rest:tt)* }) => {
        $crate::using_impl!($target $scope in_for ($for_pattern) () { $($rest)* })
    };

    ($target:ident $scope:ident in_for
        ($for_pattern:pat)
        ($($for_exp:tt)*)
        { { $($body:tt)* } $($rest:tt)* }
    ) => {
        {
            for $for_pattern in $($for_exp)* {
                $crate::using_impl!($target block empty { $($body)* })
            }
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };

    ($target:ident $scope:ident in_for
        ($for_pattern:pat)
        ($($for_exp:tt)*)
        { $t:tt $($rest:tt)* }
    ) => {
        $crate::using_impl!($target $scope in_for ($for_pattern) ($($for_exp)* $t) { $($rest)* })
    };



    ($target:ident $scope:ident empty { $st:stmt; $($rest:tt)* }) => {
        {
            $st
            $crate::using_impl!($target $scope empty { $($rest)* })
        }
    };

    ($target:ident $scope:ident empty { $exp:expr }) => {
        $exp
    };
}

#[cfg(test)]
mod tests {
    use crate::using;

    #[test]
    fn simple() {
        let vec = using!(Vec::new() => {
            .push(1);
            .push(2);
            .push(3);
            .push(4);
            .push(5);
        });
        assert_eq!(vec.iter().sum::<i32>(), 15);
    }

    #[test]
    fn simple_expr() {
        let sum = using!(Vec::new() => {
            .push(1);
            .push(2);
            .push(3);
            .push(4);
            .push(5);
            .iter().sum::<i32>()
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn block_expr() {
        let sum: i32 = using!(Vec::new() => {
            .push(1);
            {
                .push(2);
                .push(3);
            }
            .push(4);
            {
                .push(5);
                .iter().sum()
            }
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn if_expr() {
        for i in 0..3 {
            let res = using!(Vec::new() => {
                if let 0 = i {
                    .push(0);
                } else if i == 1 {
                    .push(1);
                } else {
                    .push(2);
                }
                .pop().unwrap()
            });
            assert_eq!(res, i);
        }
    }

    #[test]
    fn match_expr() {
        for i in 0..9 {
            let res = using!(vec @ Vec::new() => {
                match i {
                    0 => .push(0),
                    1 => vec.push(1),
                    2 => { .push(2) }
                    3 => { .push(3) },
                    4 if true => .push(4),
                    5 if true => vec.push(5),
                    6 if true => { .push(6) }
                    7 if true => { .push(7) },
                    _ => { .push(8) }
                }
                .pop().unwrap()
            });
            assert_eq!(res, i);
        }
    }

    #[test]
    fn loop_expr() {
        let sum: i32 = using!(Vec::new() => {
            let mut i = 1;
            loop {
                if i > 5 {
                    break;
                }
                .push(i);
                i += 1;
            }
            .iter().sum()
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn while_loop() {
        let sum: i32 = using!(Vec::new() => {
            let mut i = 1;
            while i <= 5 {
                .push(i);
                i += 1;
            }
            .iter().sum()
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn while_let() {
        let sum: i32 = using!(Vec::new() => {
            let mut i = 1;
            while let Some(_) = (i <= 5).then_some(i) {
                .push(i);
                i += 1;
            }
            .iter().sum()
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn for_loop() {
        let sum: i32 = using!(Vec::new() => {
            for i in 1..=5 {
                .push(i);
            }
            .iter().sum()
        });
        assert_eq!(sum, 15);
    }

    #[test]
    fn if_in_for() {
        let sum: i32 = using!(Vec::new() => {
            for i in 1..=10 {
                if i % 2 == 0 {
                    .push(i);
                }
            }
            .iter().sum()
        });
        assert_eq!(sum, 30);
    }

    #[test]
    fn let_exp() {
        let sum: i32 = using!(Vec::new() => {
            .push(1);
            .push(2);
            .push(3);
            let sum = .iter().sum();
            .push(sum);
            let res = { .pop().unwrap() };
            2 * res
        });
        assert_eq!(sum, 12);
    }

    #[test]
    fn let_complex() {
        let res = using!(Vec::new() => {
            .push(2);
            .push(3);
            .push(5);
            let a = loop { let x = .last().unwrap(); break *x };
            let b = if a < 10 { .first().is_some() } else { .is_empty() };
            let c = match b { true => .len(), false => 0 };
            (a, b, c)
        });
        assert_eq!(res, (5, true, 3));
    }

    #[test]
    fn nested_using() {
        let sum: i32 = using!(Vec::new() => {
            .push(1);
            .push(2);
            .push(3);
            .push(4);
            .push(5);
            .push(using!(Vec::new() => {
                .push(2);
                .push(3);
                .iter().product()
            }));
            .iter().sum()
        });
        assert_eq!(sum, 21);
    }
}
