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

#[doc(hidden)]
#[macro_export]
macro_rules! __wp_write_root {
    ($func:ident($($arg:expr),*)) => {{
        $crate::logger::sync();
        let mut root = $crate::logger::ROOT.write();
        root.$func($($arg),*)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wp_read_root {
    ($func:ident($($arg:expr),*)) => {{
        $crate::logger::LROOT.with(|root| {
            $crate::logger::uproot(&root);
            let root = root.borrow();
            root.$func($($arg),*)
        })
    }};
}
