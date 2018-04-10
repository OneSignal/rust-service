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
use std::io;
use std::sync;

use log;

/// Additional requirents for CLI options to initialize the logging subsystem
pub trait LogOptions {
    /// Print a <number> indicating syslog level with each message
    ///
    /// Allows systemd to record in system log at appropriate level.
    fn include_systemd_level(&self) -> bool {
        false
    }

    /// Only messages with this prefix will be filtered. For instance, if the
    /// package name is "foobar", returning a string "foobar" here will cause
    /// only messages from the main package to be emitted.
    fn target_filter(&self) -> String;

    /// Controls minimum level of messages to be logged.
    ///
    /// Messages lower than this level will not be printed.
    fn max_log_level(&self) -> log::LevelFilter;
}

impl<'a> LogOptions for &'a LogOptions {
    fn include_systemd_level(&self) -> bool {
        (*self).include_systemd_level()
    }

    fn target_filter(&self) -> String {
        (*self).target_filter()
    }

    fn max_log_level(&self) -> log::LevelFilter {
        (*self).max_log_level()
    }
}

pub struct Logger<T> {
    level: log::LevelFilter,
    output: sync::Mutex<T>,
    target_filter: String,
    include_systemd_level: bool,
}

impl<T: Send + io::Write> Logger<T> {
    pub fn new<O: LogOptions>(
        output: T,
        options: &O,
    ) -> Logger<io::LineWriter<T>> {
        let level = options.max_log_level();
        log::set_max_level(level);
        Logger {
            level: level,
            output: sync::Mutex::new(io::LineWriter::new(output)),
            target_filter: options.target_filter(),
            include_systemd_level: options.include_systemd_level(),
        }
    }

    /// Map a log level to a systemd level prefix
    ///
    /// Systemd can consume a leading numeric prefix in brackets to choose which
    /// system log level to record the message at. The log crate's "Info" level
    /// is mapped to "Notice" system level since they seem semantically
    /// equivalent. Warning, error, and debug levels are as expected. Both Trace
    /// and Debug are recorded at the syslog Debug level since there's no trace
    /// level.
    fn systemd_level(&self, record: &log::Record) -> &'static str {
        use ::log::Level::*;
        if self.include_systemd_level {
            match record.level() {
                Error => "<3> ",
                Warn => "<4> ",
                Info => "<5> ",
                Debug => "<7> ",
                Trace => "<7> ",
            }
        } else {
            ""
        }
    }
}

impl<T: Send + io::Write> log::Log for Logger<T> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) && record.target().starts_with(&self.target_filter) {
            let prefix = self.systemd_level(record);

            if let Ok(ref mut writer) = self.output.lock() {
                // Nothing we can do with an error here other than panic the
                // program, and that doesn't sound great either.
                let _ = writeln!(writer, "{}{}", prefix, record.args());
            }
        }
    }

    fn flush(&self) {
        if let Ok(ref mut output) = self.output.lock() {
            let _ = output.flush();
        }
    }
}

pub fn init<O: LogOptions>(options: &LogOptions) -> Result<(), log::SetLoggerError> {
    // Use env_logger if RUST_LOG environment variable is defined. Otherwise,
    // use the stdout program-only logger with optional systemd prefixing.
    if ::std::env::var("RUST_LOG").is_ok() {
        ::env_logger::try_init()
    } else {
        log::set_boxed_logger(Box::new(Logger::new(io::stdout(), &options)))
    }
}

