// Copyright 2018 OneSignal, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A framework for writing system services
//!
//! # About
//!
//! Every application which runs as a service should implement things like
//! logging, signal handling, graceful shutdown, CLI option parsing, and
//! configuration file parsing. This package provides a semi-opinionated
//! framework for doing so.
//!
//! # Usage
//!
//! There are several traits exported here including [`Application`], [`Config`],
//! [`Options`], and [`LogOptions`]. The two options traits should be implemented
//! for your CLI option loadind, [`Config`] for your config file loading, and
//! [`Application`] for your application logic.
//!
//! The primary run method is [`Application::run_once`] which is called over and
//! over again in a loop. It is provided a [`Context`] type which gives the
//! application control of when it checks for signals. Any received signals are
//! passed to [`Application::received_signal`] for handling.
//!
//! Once [`Application::run_once`] returns [`Stopping::Yes`], the main loop
//! terminates and invokes [`Application::shutdown`] before exitting.
//!
//! [`Application`]: trait.Application.html
//! [`Application::run_once`]: trait.Application.html#tymethod.run_once
//! [`Application::received_signal`]: trait.Application.html#tymethod.received_signal
//! [`Application::shutdown`]: trait.Application.html#tymethod.shutdown
//! [`Stopping::Yes`]: enum.Stopping.html#variant.Yes
//! [`Config`]: trait.Config.html
//! [`Options`]: trait.Options.html
//! [`LogOptions`]: trait.LogOptions.html
//! [`Context`]: struct.Context.html

#[macro_use] extern crate chan;
#[macro_use] extern crate log;

extern crate chan_signal;
extern crate env_logger;

/// Print a message to stderr and exit(1)
#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        ::std::process::exit(1);
    }}
}

mod application; // general app stuff
mod logging;

pub use application::{
    Application,
    Stopping,
    Config,
    Options,
    Context
};

pub use logging::LogOptions;

/// Run an Application
///
/// This should be called in your `fn main()` with something like the following.
///
/// ```rust
/// fn main() {
///     if let Err(err) = run::<MyApplication>() {
///         die!("Application encountered error: {}", err);
///     }
/// }
/// ```
///
/// CLI option loading, config loading, signal handling, and etc. are all
/// initialized automatically on the Application's behalf.
pub fn run<T>() -> Result<(), T::Err>
    where T: Application
{
    let signal = chan_signal::notify(T::signals());
    let context = Context { signal };

    let opts = T::Options::load();

    let _ = logging::init::<T::Options>(&opts);
    let config = Config::load(&opts);

    let mut app = T::new(opts, config)?;

    loop {
        if let Stopping::Yes = app.run_once(&context)? {
            break;
        }
    }

    app.shutdown()?;

    Ok(())
}
