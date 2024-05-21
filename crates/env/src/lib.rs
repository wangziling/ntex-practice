use std::{error::Error, str::FromStr};

use anyhow::{anyhow, Context};

// Copied from https://github.com/rust-lang/crates.io/blob/c03b893bf63afd0d98626ca8fcb0638eafe9f55a/crates/crates_io_env_vars/src/lib.rs

fn required<T>(res: anyhow::Result<Option<T>>, key: &str) -> anyhow::Result<T> {
    match res {
        Ok(opt) => opt.ok_or_else(|| anyhow!("Failed to find required {key} environment variable.")),
        Err(error) => Err(error),
    }
}

#[track_caller]
pub fn var(key: &str) -> anyhow::Result<Option<String>> {
    match dotenvy::var(key) {
        Ok(content) => Ok(Some(content)),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[track_caller]
pub fn var_parsed<R>(key: &str) -> anyhow::Result<Option<R>>
where
    R: FromStr,
    R::Err: Error + Send + Sync + 'static,
{
    match var(key) {
        Ok(Some(content)) => {
            Ok(Some(content.parse().with_context(|| format!("Failed to parse {key} environment variable."))?))
        }
        Ok(None) => Ok(None),
        Err(error) => Err(error),
    }
}

#[track_caller]
pub fn required_var(key: &str) -> anyhow::Result<String> {
    required(var(key), key)
}

#[track_caller]
pub fn required_var_parsed<R>(key: &str) -> anyhow::Result<R>
where
    R: FromStr,
    R::Err: Error + Send + Sync + 'static,
{
    required(var_parsed(key), key)
}

#[track_caller]
pub fn list(key: &str) -> anyhow::Result<Vec<String>> {
    let values = match var(key)? {
        Some(content) if content.is_empty() => vec![],
        Some(content) => content.split(',').map(str::trim).map(String::from).collect(),
        None => vec![],
    };

    Ok(values)
}

#[track_caller]
pub fn list_parsed<R, E, F, C>(key: &str, f: F) -> anyhow::Result<Vec<R>>
where
    F: Fn(&str) -> C,
    C: Context<R, E>,
{
    let values = match var(key)? {
        Some(content) if content.is_empty() => vec![],
        None => vec![],
        Some(content) => content
            .split(',')
            .map(str::trim)
            .map(|s| f(s).with_context(|| format!("Failed to parse value \"{s}\" of {key} environment variable.")))
            .collect::<Result<_, _>>()?,
    };

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    const TEST_VAR: &str = "WEB_ENV_VARS_TEST_VAR";

    /// A mutex to ensure that the tests don't run in parallel, since they all
    /// modify the shared environment variable.
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_var() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "test");
        assert_some_eq!(assert_ok!(var(TEST_VAR)), "test");

        std::env::remove_var(TEST_VAR);
        assert_none!(assert_ok!(var(TEST_VAR)));
    }

    #[test]
    fn test_required_var() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "test");
        assert_ok_eq!(required_var(TEST_VAR), "test");

        std::env::remove_var(TEST_VAR);
        let error = assert_err!(required_var(TEST_VAR));
        assert_eq!(error.to_string(), "Failed to find required WEB_ENV_VARS_TEST_VAR environment variable.");
    }

    #[test]
    fn test_var_parsed() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "42");
        assert_some_eq!(assert_ok!(var_parsed::<i32>(TEST_VAR)), 42);

        std::env::set_var(TEST_VAR, "test");
        let error = assert_err!(var_parsed::<i32>(TEST_VAR));
        assert_eq!(error.to_string(), "Failed to parse WEB_ENV_VARS_TEST_VAR environment variable.");

        std::env::remove_var(TEST_VAR);
        assert_none!(assert_ok!(var_parsed::<i32>(TEST_VAR)));
    }

    #[test]
    fn test_required_var_parsed() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "42");
        assert_ok_eq!(required_var_parsed::<i32>(TEST_VAR), 42);

        std::env::set_var(TEST_VAR, "test");
        let error = assert_err!(required_var_parsed::<i32>(TEST_VAR));
        assert_eq!(error.to_string(), "Failed to parse WEB_ENV_VARS_TEST_VAR environment variable.");

        std::env::remove_var(TEST_VAR);
        let error = assert_err!(required_var_parsed::<i32>(TEST_VAR));
        assert_eq!(error.to_string(), "Failed to find required WEB_ENV_VARS_TEST_VAR environment variable.");
    }

    #[test]
    fn test_list() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "test");
        assert_ok_eq!(list(TEST_VAR), vec!["test"]);

        std::env::set_var(TEST_VAR, "test, foo,   bar   ");
        assert_ok_eq!(list(TEST_VAR), vec!["test", "foo", "bar"]);

        std::env::set_var(TEST_VAR, "");
        assert_ok_eq!(list(TEST_VAR), Vec::<String>::new());

        std::env::remove_var(TEST_VAR);
        assert_ok_eq!(list(TEST_VAR), Vec::<String>::new());
    }

    #[test]
    fn test_list_parsed() {
        let _guard = MUTEX.lock().unwrap();

        std::env::set_var(TEST_VAR, "42");
        assert_ok_eq!(list_parsed(TEST_VAR, i32::from_str), vec![42]);

        std::env::set_var(TEST_VAR, "42, 1,   -53   ");
        assert_ok_eq!(list_parsed(TEST_VAR, i32::from_str), vec![42, 1, -53]);

        std::env::set_var(TEST_VAR, "42, what");
        let error = assert_err!(list_parsed(TEST_VAR, i32::from_str));
        assert_eq!(error.to_string(), "Failed to parse value \"what\" of WEB_ENV_VARS_TEST_VAR environment variable.");

        std::env::set_var(TEST_VAR, "");
        assert_ok_eq!(list_parsed(TEST_VAR, i32::from_str), Vec::<i32>::new());

        std::env::remove_var(TEST_VAR);
        assert_ok_eq!(list_parsed(TEST_VAR, i32::from_str), Vec::<i32>::new());
    }
}
