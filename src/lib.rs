// This is largely taken from the Rust distribution, with only comparatively
// minor additions and alterations. Therefore, their copyright notice follows:
//
//     Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
//     file at the top-level directory of this distribution and at
//     http://rust-lang.org/COPYRIGHT.
//
//     Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
//     http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
//     <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
//     option. This file may not be copied, modified, or distributed
//     except according to those terms.
//
// I have kept my additions under the same terms (being rather fond of MIT/Apache-2.0 myself).

//! **MOPA: My Own Personal Any.** A macro to implement all the `Any` methods on your own trait.
//!
//! You like `Any`—its ability to store any `'static` type as a trait object and then downcast it
//! back to the original type is very convenient, and in fact you need it for whatever misguided
//! reason. But it’s not enough. What you *really* want is your own trait object type with `Any`’s
//! functionality glued onto it. Maybe you have a `Person` trait and you want your people to be
//! able to do various things, but you also want to be able to conveniently downcast the person to
//! its original type, right? Alas, you can’t write a type like `Box<Person + Any>` (at present,
//! anyway). So what do you do instead? Do you give up? No, no! No, no! Enter MOPA.
//!
//! > There once was a quite friendly trait  
//! > Called `Person`, with much on its plate.  
//! >     “I need to be `Any`  
//! >     To downcast to `Benny`—  
//! > But I’m not, so I guess I’ll just wait.”
//!
//! A pitiful tale, isn’t it? Especially given that there was a bear chasing it with intent to eat
//! it. Fortunately now you can *mopafy* `Person` in three simple steps:
//!
//! 1. Add the `mopa` crate to your `Cargo.toml` as usual and your crate root like so:
//!
//!    ```rust
//!    #[macro_use]
//!    extern crate mopa_revised as mopa;
//!    # fn main() { }
//!    ```
//!
//! 2. Make `Any` (`mopa::Any`, not `std::any::Any`) a supertrait of `Person`;
//!
//! 3. Call `mopafy!` macro family to generate methods:
//!
//!    ```rust,ignore
//!    // add methods for &Person, &mut Person
//!    mopafy!(Person, core);
//!    // add methods for Box<Person>
//!    mopafy!(Person, boxed);
//!    // add methods for Arc<Person>
//!    mopafy!(Person, arc);
//!    ```
//!

#![no_std]

#[cfg(any(test, doc, feature = "std"))]
extern crate std;

/// Implementation details of the `mopafy!` macro.
#[doc(hidden)]
pub mod __ {
    pub use core::any::TypeId;
    // Option and Result are in the prelude, but they might have been overridden in the macro’s
    // scope, so we do it this way to avoid issues. (Result in particular gets overridden fairly
    // often.)
    pub use core::option::Option;
    pub use core::result::Result;

    #[cfg(feature = "std")]
    pub use std::sync::Arc;
}

/// A type to emulate dynamic typing.
///
/// This is a simple wrapper around `core::any::Any` which exists for [technical reasons][#27745].
/// Every type that implements `core::any::Any` implements this `Any`.
///
/// See the [`core::any::Any` documentation](http://doc.rust-lang.org/core/any/trait.Any.html) for
/// more details.
///
/// Any traits to be mopafied must extend this trait (e.g. `trait Person: mopa::Any { }`).
///
/// If/when [#27745] is resolved, this trait may be replaced with a simple reexport of
/// `core::any::Any`. This will be a backwards-compatible change.
///
/// [#27745]: https://github.com/rust-lang/rust/issues/27745
pub trait Any: core::any::Any {
    /// Gets the `TypeId` of `self`. UNSTABLE; do not depend on it.
    #[doc(hidden)]
    fn __get_type_id(&self) -> __::TypeId;
}

impl<T: core::any::Any> Any for T {
    fn __get_type_id(&self) -> __::TypeId {
        __::TypeId::of::<T>()
    }
}

/// The macro for implementing all the `Any` methods on your own trait.
///
/// # Instructions for use
///
/// 1. Make sure your trait extends `mopa::Any` (e.g. `trait Trait: mopa::Any { }`)
///
/// 2. Mopafy your trait (see the next subsection for specifics).
///
/// 3. …
///
/// 4. Profit!
///
/// ## Mopafication techniques
///
/// There are three ways of mopafying traits, depending on what libraries you are using.
///
/// 1. If you are a **normal person**:
///
///    ```rust
///    # #[macro_use] extern crate mopa_revised as mopa;
///    trait Trait: mopa::Any { }
///    mopafy!(Trait);
///    # fn main() { }
///    ```
///
/// 2. If you are using **libcore** but not libstd (`#![no_std]`) or liballoc, write this:
///
///    ```rust
///    # #[macro_use] extern crate mopa_revised as mopa;
///    # trait Trait: mopa::Any { }
///    mopafy!(Trait, only core);
///    # fn main() { }
///    ```
///
///    Unlike the other two techniques, this only gets you the `&Any` and `&mut Any` methods; the
///    `Box<Any>` methods require liballoc.
///
/// 3. If you are using **libcore and liballoc** but not libstd (`#![no_std]`), bring
///    `alloc::boxed::Box` into scope and use `mopafy!` as usual:
///
///    ```rust,ignore
///    # // This doctest is ignored so that it doesn't break tests on the stable/beta rustc
///    # // channels where #[feature] isn’t allowed.
///    # #![feature(alloc)]
///    # #[macro_use] extern crate mopa_revised as mopa;
///    # extern crate alloc;
///    # trait Trait: mopa::Any { }
///    use alloc::boxed::Box;
///    mopafy!(Trait);
///    # fn main() { }
///    ```
#[macro_export]
macro_rules! mopafy {
    // deprecated
    ($trait_:ident) => {
        mopafy!($trait_, core);
        mopafy!($trait_, boxed);
    };

    // deprecated
    ($trait_:ident, only core) => {
        mopafy!($trait_, core);
    };

    // Implement methods for `Box<Any>`
    ($trait_:ident, boxed) => {
        #[allow(dead_code)]
        impl $trait_ {
            /// Returns the boxed value if it is of type `T`, or `Err(Self)` if it isn't.
            #[inline]
            pub fn downcast_box<T: $trait_>(self: Box<Self>) -> $crate::__::Result<Box<T>, Box<Self>> {
                if self.is::<T>() {
                    unsafe {
                        $crate::__::Result::Ok(self.downcast_box_unchecked())
                    }
                } else {
                    $crate::__::Result::Err(self)
                }
            }

            /// Returns the boxed value, blindly assuming it to be of type `T`.
            /// If you are not *absolutely certain* of `T`, you *must not* call this.
            #[inline]
            pub unsafe fn downcast_box_unchecked<T: $trait_>(self: Box<Self>) -> Box<T> {
                Box::from_raw(Box::into_raw(self) as *mut T)
            }
        }
    };

    // Implement methods for `Arc<Any>`
    ($trait_:ident, arc) => {
        #[allow(dead_code)]
        impl $trait_ {
            #[inline]
            pub fn downcast_arc<T: $trait_>(this: $crate::__::Arc<Self>) -> $crate::__::Result<$crate::__::Arc<T>, $crate::__::Arc<Self>> {
                if this.is::<T>() {
                    unsafe {
                        $crate::__::Result::Ok($trait_::downcast_arc_unchecked(this))
                    }
                } else {
                    $crate::__::Result::Err(this)
                }
            }

            #[inline]
            pub unsafe fn downcast_arc_unchecked<T: $trait_>(this: $crate::__::Arc<Self>) -> $crate::__::Arc<T> {
                $crate::__::Arc::from_raw($crate::__::Arc::into_raw(this) as *mut T)
            }
        }
    };

    // Implement methods for `&Any` and `&mut Any`
    ($trait_:ident, core) => {
        #[allow(dead_code)]
        impl $trait_ {
            /// Returns true if the boxed type is the same as `T`
            #[inline]
            pub fn is<T: $trait_>(&self) -> bool {
                $crate::__::TypeId::of::<T>() == $crate::Any::__get_type_id(self)
            }

            /// Returns some reference to the boxed value if it is of type `T`, or
            /// `None` if it isn't.
            #[inline]
            pub fn downcast_ref<T: $trait_>(&self) -> $crate::__::Option<&T> {
                if self.is::<T>() {
                    unsafe {
                        $crate::__::Option::Some(self.downcast_ref_unchecked())
                    }
                } else {
                    $crate::__::Option::None
                }
            }

            /// Returns a reference to the boxed value, blindly assuming it to be of type `T`.
            /// If you are not *absolutely certain* of `T`, you *must not* call this.
            #[inline]
            pub unsafe fn downcast_ref_unchecked<T: $trait_>(&self) -> &T {
                &*(self as *const Self as *const T)
            }

            /// Returns some mutable reference to the boxed value if it is of type `T`, or
            /// `None` if it isn't.
            #[inline]
            pub fn downcast_mut<T: $trait_>(&mut self) -> $crate::__::Option<&mut T> {
                if self.is::<T>() {
                    unsafe {
                        $crate::__::Option::Some(self.downcast_mut_unchecked())
                    }
                } else {
                    $crate::__::Option::None
                }
            }

            /// Returns a mutable reference to the boxed value, blindly assuming it to be of type `T`.
            /// If you are not *absolutely certain* of `T`, you *must not* call this.
            #[inline]
            pub unsafe fn downcast_mut_unchecked<T: $trait_>(&mut self) -> &mut T {
                &mut *(self as *mut Self as *mut T)
            }
        }
    };
}

#[cfg(doc)]
mod example {
    use std::prelude::v1::*;

    trait Person: super::Any {
        fn weight(&self) -> i16;
    }

    mopafy!(Person, core);
    mopafy!(Person, boxed);
    mopafy!(Person, arc);
}

#[cfg(test)]
mod tests {
    use std::prelude::v1::*;

    trait Person: super::Any {
        fn weight(&self) -> i16;
    }

    mopafy!(Person, core);
    mopafy!(Person, boxed);
    mopafy!(Person, arc);

    #[derive(Clone, Debug, PartialEq)]
    struct Benny {
        // (Benny is not a superhero. He can’t carry more than 256kg of food at once.)
        kilograms_of_food: u8,
    }

    impl Person for Benny {
        fn weight(&self) -> i16 {
            self.kilograms_of_food as i16 + 60
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Chris;

    impl Person for Chris {
        fn weight(&self) -> i16 { -5 /* antigravity device! cool! */ }
    }

    #[test]
    fn test_ref() {
        let benny = Benny { kilograms_of_food: 13 };
        let benny_ptr: *const Benny = &benny;
        let person: &Person = &benny;

        assert!(person.is::<Benny>());
        assert_eq!(person.downcast_ref::<Benny>().map(|x| x as *const Benny), Some(benny_ptr));
        assert_eq!(unsafe { person.downcast_ref_unchecked::<Benny>() as *const Benny }, benny_ptr);

        assert!(!person.is::<Chris>());
        assert_eq!(person.downcast_ref::<Chris>(), None);
    }

    #[test]
    fn test_mut() {
        let mut benny = Benny { kilograms_of_food: 13 };
        let benny_ptr: *const Benny = &benny;
        let person: &mut Person = &mut benny;
        assert!(person.is::<Benny>());
        assert_eq!(person.downcast_ref::<Benny>().map(|x| x as *const Benny), Some(benny_ptr));
        assert_eq!(person.downcast_mut::<Benny>().map(|x| &*x as *const Benny), Some(benny_ptr));
        assert_eq!(unsafe { person.downcast_ref_unchecked::<Benny>() as *const Benny }, benny_ptr);
        assert_eq!(unsafe { &*person.downcast_mut_unchecked::<Benny>() as *const Benny }, benny_ptr);

        assert!(!person.is::<Chris>());
        assert_eq!(person.downcast_ref::<Chris>(), None);
        assert_eq!(person.downcast_mut::<Chris>(), None);
    }

    #[test]
    fn test_box() {
        let mut benny = Benny { kilograms_of_food: 13 };
        let mut person: Box<Person> = Box::new(benny.clone());
        assert!(person.is::<Benny>());
        assert_eq!(person.downcast_ref::<Benny>(), Some(&benny));
        assert_eq!(person.downcast_mut::<Benny>(), Some(&mut benny));
        assert_eq!(person.downcast_box::<Benny>().map(|x| *x).ok(), Some(benny.clone()));

        person = Box::new(benny.clone());
        assert_eq!(unsafe { person.downcast_ref_unchecked::<Benny>() }, &benny);
        assert_eq!(unsafe { person.downcast_mut_unchecked::<Benny>() }, &mut benny);
        assert_eq!(unsafe { *person.downcast_box_unchecked::<Benny>() }, benny);

        person = Box::new(benny.clone());
        assert!(!person.is::<Chris>());
        assert_eq!(person.downcast_ref::<Chris>(), None);
        assert_eq!(person.downcast_mut::<Chris>(), None);
        assert!(person.downcast_box::<Chris>().err().is_some());
    }

    #[test]
    fn test_arc() {
        use std::sync::Arc;

        let benny = Benny { kilograms_of_food: 13 };
        let person: Arc<Person> = Arc::new(benny.clone());
        let person1 = person.clone();
        let person2 = person.clone();
        assert!(person.is::<Benny>());
        assert!(person1.is::<Benny>());
        assert!(person2.is::<Benny>());
        assert_eq!(Arc::strong_count(&person), 3);
        assert_eq!(person.downcast_ref::<Benny>(), Some(&benny));
        {
            let b2 = Person::downcast_arc::<Benny>(person).ok().unwrap();
            assert_eq!(b2.as_ref(), &benny);
            assert_eq!(Arc::strong_count(&b2), 3);
        }
        assert_eq!(Arc::strong_count(&person1), 2);
        assert_eq!(person1.downcast_ref::<Benny>(), Some(&benny));
        assert_eq!(person2.downcast_ref::<Benny>(), Some(&benny));
    }
}
