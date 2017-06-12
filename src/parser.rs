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

extern crate serde_json;
use self::serde_json::Value;

use std;

use levels::LogLevel;
use line_range::{LineRangeSpec, prepare_ranges};

#[derive(PartialEq, PartialOrd, Clone, Debug)]
struct Module {
    path: String,
    level: Option<LogLevel>,
    lranges: Option<Vec<LineRangeSpec>>,
}

impl Module {
    fn with_level(path: &str, level: LogLevel) -> Self {
        Module {
            path: path.to_string(),
            level: Some(level),
            lranges: None,
        }
    }

    fn with_lranges(path: &str, level: LogLevel, lranges: &Vec<(u32, u32)>) -> Result<Self, String> {
        Ok(Module {
            path: path.to_string(),
            level: None,
            lranges: Some(prepare_ranges(level, &lranges)?),
        })
    }
}

#[doc(hidden)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Root {
    level: Option<LogLevel>,
    modules: Vec<Module>,
}

impl Root {
    fn new() -> Self {
        Root {
            level: None,
            modules: Vec::new(),
        }
    }

    #[cfg(test)]
    fn with_level(level: LogLevel) -> Self {
        Root {
            level: Some(level),
            modules: Vec::new(),
        }
    }

    #[cfg(test)]
    fn module(mut self, module: Module) -> Self {
        self.modules.push(module);
        self
    }
}

/// Represents log spec parse failure
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum JsonError {
    /// Invalid JSON string
    Json,
    /// The root is invalid
    Root,
    /// The array of modules is invalid
    Module,
    /// The log level of the root is invalid
    RootLogLevel,
    /// The log level of the module is invalid
    ModuleLogLevel,
    /// The line range of the module is invalid
    LineRange,
}

/// Represents log spec parse failure
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum ParseError {
    /// Invalid spec
    Spec,
    /// Invalid log level
    LogLevel,
    /// Invalid line range
    LineRange(String),
    /// JSON parse error
    Json(JsonError),
}

fn parse_json(json: &str) -> Result<Root, ParseError> {
    let spec: Value = serde_json::from_str(json)
        .or(Err(ParseError::Json(JsonError::Json)))?;
    let spec = spec.as_object()
        .ok_or(ParseError::Json(JsonError::Root))?;
    if spec.is_empty() {
        return Err(ParseError::Json(JsonError::Root));
    }

    let mut root = Root::new();

    if let Some(level) = spec.get("level") {
        root.level = Some(
            level.as_str()
                .ok_or(ParseError::Json(JsonError::RootLogLevel))?.parse()
                .or(Err(ParseError::Json(JsonError::RootLogLevel)))?
        );
    }

    if let Some(modules) = spec.get("modules") {
        let modules = modules.as_array()
            .ok_or(ParseError::Json(JsonError::Module))?;
        if modules.is_empty() {
            return Err(ParseError::Json(JsonError::Module));
        }
        for module in modules {
            let module = module.as_object()
                .ok_or(ParseError::Json(JsonError::Module))?;
            let path = module.get("path")
                .ok_or(ParseError::Json(JsonError::Module))?.as_str()
                .ok_or(ParseError::Json(JsonError::Module))?;
            let level = if let Some(level) = module.get("level") {
                level.as_str()
                    .ok_or(ParseError::Json(JsonError::ModuleLogLevel))?.parse()
                    .or(Err(ParseError::Json(JsonError::ModuleLogLevel)))?
            } else {
                LogLevel::TRACE
            };

            let module = if let Some(lines) = module.get("lines") {
                let lines = lines.as_array()
                    .ok_or(ParseError::Json(JsonError::LineRange))?;
                let mut lranges = Vec::new();
                if lines.is_empty() {
                    return Err(ParseError::Json(JsonError::LineRange));
                }
                for line in lines {
                    let line = line.as_array()
                        .ok_or(ParseError::Json(JsonError::LineRange))?;
                    if line.len() != 2 {
                        return Err(ParseError::Json(JsonError::LineRange));
                    }
                    let from = line[0].as_u64()
                        .ok_or(ParseError::Json(JsonError::LineRange))?;
                    if from > std::u32::MAX as u64 {
                        return Err(ParseError::Json(JsonError::LineRange))
                    }
                    let to = line[1].as_u64()
                        .ok_or(ParseError::Json(JsonError::LineRange))?;
                    if to > std::u32::MAX as u64 {
                        return Err(ParseError::Json(JsonError::LineRange))
                    }
                    lranges.push((from as u32, to as u32));
                }
                Module::with_lranges(path, level, &lranges)
                    .or(Err(ParseError::Json(JsonError::LineRange)))?
            } else {
                Module::with_level(path, level)
            };
            root.modules.push(module);
        }
    }

    Ok(root)
}

fn parse_token(root: &mut Root, token: &str) -> Result<(), ParseError> {
    if token.is_empty() {
        return Err(ParseError::Spec);
    }

    let mut kv = token.split('=');
    let (k, v) = (kv.next(), kv.next());

    if kv.next().is_some() {
        return Err(ParseError::Spec);
    }

    if k.is_none() {
        return Err(ParseError::Spec);
    }
    let k = k.unwrap().trim();
    if k.is_empty() {
        return Err(ParseError::Spec);
    }

    if v.is_none() {
        // `k` is either global log level or path for which log level is `TRACE`
        if let Ok(level) = k.to_uppercase().parse() {
            root.level = Some(level);
        } else {
            root.modules.push(Module::with_level(k, LogLevel::TRACE));
        }
    } else {
        let v = v.unwrap().trim();
        if v.is_empty() {
            return Err(ParseError::Spec);
        }

        if let Ok(level) = v.to_uppercase().parse() {
            root.modules.push(Module::with_level(k, level));
        } else {
            return Err(ParseError::Spec);
        }
    }

    Ok(())
}

pub fn parse(spec: &str) -> Result<Root, ParseError> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err(ParseError::Spec);
    }

    if spec.starts_with('{') {
        return parse_json(spec);
    }

    let mut root = Root::new();
    for token in spec.split(',') {
        parse_token(&mut root, token)?;
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_invalid() {
        assert_eq!(Err(ParseError::Spec), parse(""));
        assert_eq!(Err(ParseError::Spec), parse(","));
        assert_eq!(Err(ParseError::Spec), parse("="));
        assert_eq!(Err(ParseError::Spec), parse("foo="));
        assert_eq!(Err(ParseError::Spec), parse("=error"));
        assert_eq!(Err(ParseError::Spec), parse("foo,"));
        assert_eq!(Err(ParseError::Spec), parse(",foo"));
        assert_eq!(Err(ParseError::Spec), parse("foo=bar"));
    }

    #[test]
    fn test_spec_global() {
        let expect = Root::with_level(LogLevel::ERROR);
        assert_eq!(Ok(expect), parse("error"));
    }

    #[test]
    fn test_spec_module() {
        let expect = Root::new()
            .module(Module::with_level("foo", LogLevel::ERROR));
        assert_eq!(Ok(expect), parse("foo=error"));

        let expect = Root::new()
            .module(Module::with_level("foo", LogLevel::TRACE));
        assert_eq!(Ok(expect), parse("foo"));
    }

    #[test]
    fn test_spec_combined() {
        let expect = Root::with_level(LogLevel::CRITICAL)
            .module(Module::with_level("foo", LogLevel::TRACE))
            .module(Module::with_level("bar", LogLevel::TRACE))
            .module(Module::with_level("qux", LogLevel::ERROR));
        assert_eq!(Ok(expect), parse("critical,foo,bar,qux=error"));
    }

    #[test]
    fn test_spec_json() {
        let expect = Root::with_level(LogLevel::CRITICAL)
            .module(Module::with_level("foo", LogLevel::TRACE))
            .module(Module::with_level("bar", LogLevel::TRACE))
            .module(Module::with_lranges("bar", LogLevel::ERROR,
                                       &vec!((10, 100), (120, 130))).unwrap());
        assert_eq!(Ok(expect), parse(r#"{
                                            "level": "critical",
                                            "modules": [
                                                {
                                                    "path": "foo"
                                                },
                                                {
                                                    "path": "bar"
                                                },
                                                {
                                                    "path": "bar",
                                                    "level": "error",
                                                    "lines": [
                                                        [10, 100], [120, 130]
                                                    ]
                                                }
                                            ]
                                        }"#));
    }

    #[test]
    fn test_spec_json_invalid() {
        assert_eq!(Err(ParseError::Json(JsonError::Json)), parse("{"));

        assert_eq!(Err(ParseError::Json(JsonError::Root)),
                   parse(r#"{}"#));

        assert_eq!(Err(ParseError::Json(JsonError::RootLogLevel)),
                   parse(r#"{"level": "foo"}"#));

        assert_eq!(Err(ParseError::Json(JsonError::Module)),
                   parse(r#"{"modules": {}}"#));
        assert_eq!(Err(ParseError::Json(JsonError::Module)),
                   parse(r#"{"modules": []}"#));
        assert_eq!(Err(ParseError::Json(JsonError::Module)),
                   parse(r#"{"modules": [{"level": "critical"}]}"#));

        assert_eq!(Err(ParseError::Json(JsonError::ModuleLogLevel)),
                   parse(r#"{"modules": [{"path": "bar", "level": "foo"}]}"#));

        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": {}}]}"#));
        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": []}]}"#));
        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": [[]]}]}"#));
        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": [[1]]}]}"#));
        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": [[1, 10, 20]]}]}"#));
        assert_eq!(Err(ParseError::Json(JsonError::LineRange)),
                   parse(r#"{"modules": [{"path": "bar", "lines": [[10, 1]]}]}"#));
    }
}