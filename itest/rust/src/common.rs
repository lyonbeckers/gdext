/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Debug;

use godot::meta::{FromGodot, ToGodot};

pub fn roundtrip<T>(value: T)
where
    T: FromGodot + ToGodot + PartialEq + Debug,
{
    // TODO test other roundtrip (first FromGodot, then ToGodot)
    // Some values can be represented in Variant, but not in T (e.g. Variant(0i64) -> Option<InstanceId> -> Variant is lossy)

    let variant = value.to_variant();
    let back = T::try_from_variant(&variant).unwrap();

    assert_eq!(value, back);
}

/// Signal to the compiler that a value is used (to avoid optimization).
pub fn bench_used<T: Sized>(value: T) {
    // The following check would be used to prevent `()` arguments, ensuring that a value from the bench is actually going into the blackbox.
    // However, we run into this issue, despite no array being used: https://github.com/rust-lang/rust/issues/43408.
    //   error[E0401]: can't use generic parameters from outer function
    // sys::static_assert!(std::mem::size_of::<T>() != 0, "returned unit value in benchmark; make sure to use a real value");

    std::hint::black_box(value);
}

#[cfg(since_api = "4.5")]
mod itest_error_logger {
    use std::sync::atomic::{AtomicBool, Ordering};

    use godot::builtin::{Array, GString};
    use godot::classes::{ILogger, Logger, Os, ScriptBacktrace};
    use godot::meta::GodotType;
    use godot::obj::{Base, Gd, Singleton};
    use godot::register::{godot_api, GodotClass};

    #[derive(GodotClass)]
    #[class(no_init, base = Logger)]
    pub struct ItestErrorLogger {
        expected_error: GString,
        was_error_called: AtomicBool,
        base: Base<Logger>,
    }

    impl ItestErrorLogger {
        pub fn was_error_called(&self) -> bool {
            self.was_error_called.fetch_or(false, Ordering::Relaxed)
        }
    }

    #[godot_api]
    impl ILogger for ItestErrorLogger {
        fn log_error(
            &mut self,
            _function: GString,
            _file: GString,
            _line: i32,
            code: GString,
            _rationale: GString,
            _editor_notify: bool,
            _error_type: i32,
            _backtrace: Array<Gd<ScriptBacktrace>>,
        ) {
            if code == self.expected_error {
                self.was_error_called.store(true, Ordering::Relaxed)
            }
        }
    }

    pub fn itest_logger(expected_error: GString) -> Gd<ItestErrorLogger> {
        // We need to register ScriptBacktrace ClassID beforehand – otherwise it might be tried to be initialized from different thread (or two threads at once).
        let _script_backtrace = Gd::<ScriptBacktrace>::class_id();
        let logger = Gd::from_init_fn(|base| ItestErrorLogger {
            expected_error,
            was_error_called: AtomicBool::new(false),
            base,
        });
        Os::singleton().add_logger(&logger);

        logger
    }
}

#[allow(unused)]
#[cfg(since_api = "4.5")]
pub use itest_error_logger::*;
