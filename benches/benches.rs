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

#[macro_use]
extern crate bencher;

#[macro_use]
extern crate woodpecker;

mod wpb {
    use bencher::Bencher;
    use woodpecker as wp;

    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Once, ONCE_INIT};

    static ONCE: Once = ONCE_INIT;
    const THREADS_QTY: usize = 4;
    const FOO_LOGGERS_QTY: usize = 10;

    struct LThreads {
        threads: Vec<thread::JoinHandle<()>>,
        stop: Arc<AtomicBool>,
    }

    impl LThreads {
        fn new(f: Arc<Fn() + Sync + Send>) -> Self {
            let mut threads = Vec::new();
            let stop = Arc::new(AtomicBool::new(false));
            for _ in 0..THREADS_QTY {
                let stop = stop.clone();
                let f = f.clone();
                threads.push(thread::spawn(move || {
                    while !stop.load(Ordering::Acquire) {
                        thread::yield_now();
                        f();
                    }
                }));
            }
            LThreads {
                threads: threads,
                stop: stop,
            }
        }

        fn join(&mut self) {
            self.stop.store(true, Ordering::Release);
            for t in self.threads.drain(..) {
                t.join().unwrap();
            }
        }
    }

    impl Drop for LThreads {
        fn drop(&mut self) {
            self.join();
        }
    }

    macro_rules! jail {
        ($b:ident, $body:expr) => {
            reset();
            $b.iter(|| { $body; });
        };
        ($b:ident, $pre:expr, $body:expr) => {
            reset();
            $pre;
            $b.iter(|| { $body; });
        };
    }

    macro_rules! tjail {
        ($b:ident, $body:expr) => {
            reset();
            let t = LThreads::new(Arc::new(|| { $body; }));
            $b.iter(|| { $body; });
            drop(t);
        };
        ($b:ident, $pre:expr, $body:expr) => {
            reset();
            $pre;
            let t = LThreads::new(Arc::new(|| { $body; }));
            $b.iter(|| { $body; });
            drop(t);
        };
    }

    macro_rules! doutput {
        () => { debug!("{} -> {}", "test", 1.0) }
    }

    macro_rules! coutput {
        () => { critical!("{} -> {}", "test", 1.0) }
    }

    fn reset() {
        ONCE.call_once(|| {
            let config = wp::Config {
                thread: cfg!(feature = "test-thread-log"),
            };
            wp_init!(&config);
        });
        wp::logger::reset();
        wp_set_level!(wp::LogLevel::ERROR).unwrap();
    }

    fn drop_output() {
        wp_register_handler!(Box::new(|_| {}));
    }

    fn drop_formatted_output() {
        wp_register_handler!(Box::new(|r| {
            let _ = r.formatted();
        }));
    }

    fn foo_loggers() {
        for idx in 0..FOO_LOGGERS_QTY {
            wp_set_level!(wp::LogLevel::DEBUG, &format!("foo::bar::qux::{}", idx)).unwrap();
        }
    }

    fn foo_loggers_this_module() {
        foo_loggers();
        wp_set_level!(wp::LogLevel::INFO, this_module!()).unwrap();
    }

    fn foo_loggers_this_file() {
        foo_loggers();
        wp_set_level!(wp::LogLevel::INFO, this_file!()).unwrap();
    }

    fn foo_loggers_this_file_with_lines() {
        foo_loggers();
        let (bof, eof): (u32, u32) = (wp::BOF.into(), wp::EOF.into());
        wp_set_level!(wp::LogLevel::CRITICAL, this_file!(), [(bof + 1, bof + 10)]).unwrap();
        wp_set_level!(wp::LogLevel::INFO, this_file!(), [(bof + 11, eof - 1)]).unwrap();
    }

    // No output, single thread
    fn bench_no_output_single_thread(b: &mut Bencher) {
        jail!(
            b,
            doutput!()
        );
    }

    fn bench_no_output_sub_other_single_thread(b: &mut Bencher) {
        jail!(
            b,
            foo_loggers(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_module_single_thread(b: &mut Bencher) {
        jail!(
            b,
            foo_loggers_this_module(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_file_single_thread(b: &mut Bencher) {
        jail!(
            b,
            foo_loggers_this_file(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_file_with_lines_single_thread(b: &mut Bencher) {
        jail!(
            b,
            foo_loggers_this_file_with_lines(),
            doutput!()
        );
    }

    // No output, multi thread
    fn bench_no_output_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            doutput!()
        );
    }

    fn bench_no_output_sub_other_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            foo_loggers(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_module_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            foo_loggers_this_module(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_file_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            foo_loggers_this_file(),
            doutput!()
        );
    }

    fn bench_no_output_sub_this_file_with_lines_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            foo_loggers_this_file_with_lines(),
            doutput!()
        );
    }

    // Drop output, single thread
    fn bench_drop_output_single_thread(b: &mut Bencher) {
        jail!(
            b,
            drop_output(),
            coutput!()
        );
    }

    fn bench_drop_output_sub_other_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_output();
                foo_loggers();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_module_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_output();
                foo_loggers_this_module();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_file_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_output();
                foo_loggers_this_file();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_file_with_lines_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_output();
                foo_loggers_this_file_with_lines();
            },
            coutput!()
        );
    }

    // Drop output, multi thread
    fn bench_drop_output_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            drop_output(),
            coutput!()
        );
    }

    fn bench_drop_output_sub_other_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_output();
                foo_loggers();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_module_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_output();
                foo_loggers_this_module();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_file_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_output();
                foo_loggers_this_file();
            },
            coutput!()
        );
    }

    fn bench_drop_output_sub_this_file_with_lines_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_output();
                foo_loggers_this_file_with_lines();
            },
            coutput!()
        );
    }

    // Drop formatted output, single thread
    fn bench_drop_formatted_output_single_thread(b: &mut Bencher) {
        jail!(
            b,
            drop_formatted_output(),
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_other_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_formatted_output();
                foo_loggers();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_module_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_module();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_file_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_file();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_file_with_lines_single_thread(b: &mut Bencher) {
        jail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_file_with_lines();
            },
            coutput!()
        );
    }

    // Drop formatted output, multi thread
    fn bench_drop_formatted_output_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            drop_formatted_output(),
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_other_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_formatted_output();
                foo_loggers();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_module_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_module();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_file_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_file();
            },
            coutput!()
        );
    }

    fn bench_drop_formatted_output_sub_this_file_with_lines_multi_thread(b: &mut Bencher) {
        tjail!(
            b,
            {
                drop_formatted_output();
                foo_loggers_this_file_with_lines();
            },
            coutput!()
        );
    }

    fn bench_stub(_: &mut Bencher) {
    }

    benchmark_group!(
        benches,

        bench_no_output_single_thread,
        bench_no_output_sub_this_file_single_thread,
        bench_no_output_sub_this_module_single_thread,
        bench_no_output_sub_this_file_with_lines_single_thread,
        bench_no_output_sub_other_single_thread,

        bench_drop_output_single_thread,
        bench_drop_output_sub_this_module_single_thread,
        bench_drop_output_sub_this_file_single_thread,
        bench_drop_output_sub_this_file_with_lines_single_thread,
        bench_drop_output_sub_other_single_thread,

        bench_drop_formatted_output_single_thread,
        bench_drop_formatted_output_sub_this_module_single_thread,
        bench_drop_formatted_output_sub_this_file_single_thread,
        bench_drop_formatted_output_sub_this_file_with_lines_single_thread,
        bench_drop_formatted_output_sub_other_single_thread,

        bench_no_output_multi_thread,
        bench_no_output_sub_this_module_multi_thread,
        bench_no_output_sub_this_file_multi_thread,
        bench_no_output_sub_this_file_with_lines_multi_thread,
        bench_no_output_sub_other_multi_thread,

        bench_drop_output_multi_thread,
        bench_drop_output_sub_this_module_multi_thread,
        bench_drop_output_sub_this_file_multi_thread,
        bench_drop_output_sub_this_file_with_lines_multi_thread,
        bench_drop_output_sub_other_multi_thread,

        bench_drop_formatted_output_multi_thread,
        bench_drop_formatted_output_sub_this_module_multi_thread,
        bench_drop_formatted_output_sub_this_file_multi_thread,
        bench_drop_formatted_output_sub_this_file_with_lines_multi_thread,
        bench_drop_formatted_output_sub_other_multi_thread,

        bench_stub
    );
}
benchmark_main!(wpb::benches);
