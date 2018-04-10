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
//! General types applicable to any Application
use std::path::Path;
use std::borrow::Cow;

use chan::Receiver;
use chan_signal::Signal;

use logging::LogOptions;

/// Indicates whether the run loop should halt
pub enum Stopping {
    /// The run loop should halt
    Yes,

    /// The run loop should continue
    No
}

/// Trait required for loading Config from file
pub trait Config {
    /// Parse options and return it
    ///
    /// Implementers should use the `die!` macro to handle failure.
    fn load<O: Options>(_: &O) -> Self;
}

/// Trait required for loading CLI options
pub trait Options : LogOptions {
    /// Parse options and return them
    ///
    /// Implementers should use the `die!` macro to handle failure.
    fn load() -> Self;

    /// Get the path to a config file
    ///
    /// This method is needed since Config::load is expected to be generic.
    fn config_path<'a>(&'a self) -> Cow<'a, Path>;
}

/// A context passed to `Application::run_once`
///
/// Gives the application control over when to execute certain operations like
/// signal handling.
pub struct Context {
    pub(crate) signal: Receiver<Signal>
}

impl Context {
    pub fn poll_signals<A: Application>(&self, app: &mut A) {
        let signal = &self.signal;

        // Handle any and all pending signals.
        loop {
            chan_select! {
                default => { break; },
                signal.recv() -> sig => {
                    debug!("Received signal: {:?}", sig);
                    sig.map(|s| app.received_signal(s));
                },
            }
        }
    }
}

/// The application; domain-specific program logic
pub trait Application: Sized {
    /// Main error export of the Application
    type Err: Send + 'static;

    /// Config to be loaded from a file
    type Config: Config;

    /// Options from the command line
    type Options: Options;

    /// Create a new instance given the options and config
    fn new(_: Self::Options, _: Self::Config) -> Result<Self, Self::Err>;

    /// Called repeatedly in the main loop of the application.
    fn run_once(&mut self, context: &Context) -> Result<Stopping, Self::Err>;

    /// Which signal the application is interested in receiving.
    ///
    /// By default, only INT and TERM are blocked and handled.
    fn signals() -> &'static [Signal] {
        static SIGNALS: &[Signal] = &[Signal::INT, Signal::TERM];
        SIGNALS
    }

    /// Handle a received signal
    fn received_signal(&mut self, _: Signal) {
        die!("received_signal default action");
    }

    /// Called when the application is shutting down
    fn shutdown(self) -> Result<(), Self::Err> {
        Ok(())
    }
}
