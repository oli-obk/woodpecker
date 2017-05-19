// Copyright 2017 Dmytro Milinevskyi <dmilinevskyi@gmail.com>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate chrono;
use self::chrono::prelude::*;

extern crate time;

extern crate parking_lot;
use self::parking_lot::Mutex;

use std::sync::Arc;

use std::fmt;
use std::fmt::Write;

use formatters::Formatter;
use levels::LogLevel;
use record::Record;

#[derive(Clone)]
struct RecordMeta {
    level: LogLevel,
    module: &'static str,
    file: &'static str,
    line: u32,
    ts: time::Timespec,
}

impl RecordMeta {
    #[inline(always)]
    fn new(level: LogLevel,
           module: &'static str,
           file: &'static str,
           line: u32,
           ts: time::Timespec) -> Self {
        RecordMeta {
            level: level,
            module: module,
            file: file,
            line: line,
            ts: ts,
        }
    }
}

struct RecordLazyMetaInner {
    msg: Option<Arc<String>>,
    formatted: Option<Arc<String>>,
    ts_utc: Option<Arc<DateTime<UTC>>>,
}

impl RecordLazyMetaInner {
    #[inline(always)]
    fn new() -> Self {
        RecordLazyMetaInner {
            msg: None,
            formatted: None,
            ts_utc: None,
        }
    }

    fn mk_msg<'a>(&mut self, args: &fmt::Arguments<'a>) {
        if self.msg.is_none() {
            let mut mstr = String::new();
            mstr.write_fmt(*args).unwrap();
            self.msg = Some(Arc::new(mstr));
        }
    }

    fn mk_ts_utc(&mut self, record: &RecordMeta) {
        if self.ts_utc.is_none() {
            let naive = chrono::NaiveDateTime::from_timestamp(record.ts.sec, record.ts.nsec as u32);
            self.ts_utc = Some(Arc::new(chrono::DateTime::from_utc(naive, chrono::UTC)));
        }
    }
}

struct RecordLazyMeta {
    irecord: Mutex<RecordLazyMetaInner>,
    formatter: Arc<Formatter>,
}

impl RecordLazyMeta {
    #[inline(always)]
    fn new(formatter: Arc<Formatter>) -> Self {
        RecordLazyMeta {
            irecord: Mutex::new(RecordLazyMetaInner::new()),
            formatter: formatter,
        }
    }

    fn msg<'a>(&self, args: &fmt::Arguments<'a>) -> Arc<String> {
        let mut irecord = self.irecord.lock();
        irecord.mk_msg(args);
        let msg = irecord.msg.as_ref().unwrap();
        msg.clone()
    }

    fn formatted(&self, record: &Record) -> Arc<String> {
        {
            let irecord = self.irecord.lock();
            let format = irecord.formatted.is_none();
            drop(irecord);

            if format {
                let formatted = Arc::new((self.formatter)(record));
                let mut irecord = self.irecord.lock();
                irecord.formatted = Some(formatted.clone());
                return formatted;
            }
        }

        let irecord = self.irecord.lock();
        let formatted = irecord.formatted.as_ref().unwrap();
        formatted.clone()
    }

    fn ts_utc(&self, record: &RecordMeta) -> Arc<DateTime<UTC>> {
        let mut irecord = self.irecord.lock();
        irecord.mk_ts_utc(record);
        let ts_utc = irecord.ts_utc.as_ref().unwrap();
        ts_utc.clone()
    }
}

// TODO: use pub(crate) when stabilized (should in v1.18)
// https://github.com/rust-lang/rust/issues/32409
#[doc(hidden)]
pub struct SyncRecord<'a> {
    irecord: RecordMeta,
    args: fmt::Arguments<'a>,
    precord: Arc<RecordLazyMeta>,
}

impl<'a> SyncRecord<'a> {
    // TODO: use pub(crate) when stabilized (should in v1.18)
    // https://github.com/rust-lang/rust/issues/32409
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(level: LogLevel, module: &'static str, file: &'static str, line: u32,
           ts: time::Timespec, args: fmt::Arguments<'a>,
           formatter: Arc<Formatter>) -> Self {
        SyncRecord {
            irecord: RecordMeta::new(level, module, file, line, ts),
            args: args,
            precord: Arc::new(RecordLazyMeta::new(formatter)),
        }
    }
}

impl<'a> Record for SyncRecord<'a> {
    #[inline(always)]
    fn level(&self) -> LogLevel {
        self.irecord.level
    }

    #[inline(always)]
    fn module(&self) -> &'static str {
        self.irecord.module
    }

    #[inline(always)]
    fn file(&self) -> &'static str {
        self.irecord.file
    }

    #[inline(always)]
    fn line(&self) -> u32 {
        self.irecord.line
    }

    #[inline(always)]
    fn ts(&self) -> time::Timespec {
        self.irecord.ts
    }

    fn msg(&self) -> Arc<String> {
        return self.precord.msg(&self.args)
    }

    fn formatted(&self) -> Arc<String> {
        return self.precord.formatted(self)
    }

    fn ts_utc(&self) -> Arc<DateTime<UTC>> {
        return self.precord.ts_utc(&self.irecord)
    }
}

// TODO: use pub(crate) when stabilized (should in v1.18)
// https://github.com/rust-lang/rust/issues/32409
#[doc(hidden)]
pub struct AsyncRecord {
    irecord: RecordMeta,
    msg: Arc<String>,
    precord: Arc<RecordLazyMeta>,
}

impl Record for AsyncRecord {
    #[inline(always)]
    fn level(&self) -> LogLevel {
        self.irecord.level
    }

    #[inline(always)]
    fn module(&self) -> &'static str {
        self.irecord.module
    }

    #[inline(always)]
    fn file(&self) -> &'static str {
        self.irecord.file
    }

    #[inline(always)]
    fn line(&self) -> u32 {
        self.irecord.line
    }

    #[inline(always)]
    fn ts(&self) -> time::Timespec {
        self.irecord.ts
    }

    fn msg(&self) -> Arc<String> {
        self.msg.clone()
    }

    fn formatted(&self) -> Arc<String> {
        return self.precord.formatted(self)
    }

    fn ts_utc(&self) -> Arc<DateTime<UTC>> {
        return self.precord.ts_utc(&self.irecord)
    }
}

impl<'a> From<SyncRecord<'a>> for AsyncRecord {
    #[inline(always)]
    fn from(orig: SyncRecord) -> AsyncRecord {
        let mut mstr = String::new();
        mstr.write_fmt(orig.args).unwrap();
        AsyncRecord {
            irecord: orig.irecord,
            msg: Arc::new(mstr),
            precord: orig.precord,
        }
    }
}