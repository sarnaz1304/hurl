/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2024 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use crate::cli::options::CliOptions;
use crate::cli::CliError;
use crate::{cli, HurlRun};
use hurl::runner::{HurlResult, Input};
use hurl::{output, runner};
use std::path::Path;

/// Runs Hurl `files` sequentially, given a current directory and command-line options (see
/// [`crate::cli::options::CliOptions`]). This function returns a list of [`HurlRun`] results or
/// an error.
pub fn run_seq(
    files: &[Input],
    current_dir: &Path,
    options: &CliOptions,
) -> Result<Vec<HurlRun>, CliError> {
    let mut runs = vec![];

    for (current, filename) in files.iter().enumerate() {
        let content = filename.read_to_string()?;
        let total = files.len();
        let variables = &options.variables;
        let runner_options = options.to_runner_options(filename, current_dir);
        let logger_options = options.to_logger_options(filename, current, total);

        // Run our Hurl file now, we can only fail if there is a parsing error.
        // The parsing error is displayed in the `execute` call, that's why we gobble the error
        // string.
        let Ok(hurl_result) = runner::run(&content, &runner_options, variables, &logger_options)
        else {
            return Err(CliError::Parsing);
        };

        let success = hurl_result.success;

        // We can output the result, either the raw body or a structured JSON representation.
        let output_body = success
            && !options.interactive
            && matches!(options.output_type, cli::OutputType::ResponseBody);
        if output_body {
            if let Some(last_entry) = hurl_result.entries.last() {
                let include_headers = options.include;
                let result =
                    output::write_body(last_entry, include_headers, options.color, &options.output);
                if let Err(e) = result {
                    return Err(CliError::Runtime(e.to_string()));
                }
            }
        }
        if matches!(options.output_type, cli::OutputType::Json) {
            let result =
                output::write_json(&hurl_result, &content, filename, options.output.as_ref());
            if let Err(e) = result {
                return Err(CliError::Runtime(e.to_string()));
            }
        }

        let run = HurlRun {
            content,
            filename: filename.to_string(),
            hurl_result,
        };
        runs.push(run);
    }

    Ok(runs)
}

#[allow(unused)]
pub fn run_par(files: &[String]) -> Result<Vec<HurlResult>, CliError> {
    todo!()
}
