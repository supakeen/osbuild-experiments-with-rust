/// Version 1 of the manifest description This is the first version of the osbuild manifest description,
/// that has a main pipeline that consists of zero or more stages to create a tree and optionally one assembler that assembles
/// the created tree into an artefact. The pipeline can have any number of nested build pipelines. A sources section is used
/// to fetch resources.
pub mod v1;

/// Version 2 of manifest descriptions, this version is current.
pub mod v2;

#[derive(Debug)]
pub enum ManifestDescriptionError {}

pub enum ValidationPath {
    Name(String),
    Index(usize),
}

/// Describes a single failed validation. Consists of a `message` describing the error and a `path`
/// that points to the thing that caused the error.
pub struct ValidationError {
    message: String,
    path: Vec<ValidationPath>,
}

impl ValidationError {
    fn id(self) -> String {
        if self.path.is_empty() {
            ".".to_string()
        } else {
            let mut result = String::new();

            for part in self.path.into_iter() {
                match part {
                    ValidationPath::Name(path) => {
                        if path.contains(' ') {
                            result = format!("{}.'{}'", result, path);
                        } else {
                            result = format!("{}.{}", result, path);
                        }
                    }
                    ValidationPath::Index(path) => {
                        result = format!("{}[{}]", result, path);
                    }
                }
            }

            result
        }
    }
}

pub struct ValidationResult {
    origin: Option<String>,
    errors: Vec<ValidationError>,
}

impl ValidationResult {
    fn new(origin: String) -> Self {
        Self {
            origin: Some(origin),
            errors: vec![],
        }
    }

    /// Add a `ValidationError` to the set of errors
    fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a new `ValidationError` with `message` as message
    fn fail(&mut self, message: String) {
        self.add(ValidationError {
            message,
            path: Vec::new(),
        });
    }

    /// Merge all errors of `result` into this `ValidationResult` adjusting their paths by
    /// pre-pending the optionally supplied `path`
    fn merge(&mut self, result: ValidationResult, path: Vec<ValidationPath>) {
        for error in result.errors {
            self.add(error);
        }
    }
}

impl From<ValidationResult> for bool {
    fn from(object: ValidationResult) -> bool {
        object.errors.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validation_error_id() {
        let test0 = ValidationError {
            message: String::new(),
            path: vec![ValidationPath::Name("foo".to_string())],
        };

        assert_eq!(test0.id(), ".foo".to_string());

        let test1 = ValidationError {
            message: String::new(),
            path: vec![
                ValidationPath::Name("foo".to_string()),
                ValidationPath::Name("bar".to_string()),
            ],
        };

        assert_eq!(test1.id(), ".foo.bar".to_string());

        let test2 = ValidationError {
            message: String::new(),
            path: vec![
                ValidationPath::Name("foo".to_string()),
                ValidationPath::Name("bar".to_string()),
                ValidationPath::Index(1337),
            ],
        };

        assert_eq!(test2.id(), ".foo.bar[1337]".to_string());

        let test3 = ValidationError {
            message: String::new(),
            path: vec![
                ValidationPath::Name("foo".to_string()),
                ValidationPath::Index(42),
                ValidationPath::Name("bar".to_string()),
                ValidationPath::Index(1337),
            ],
        };

        assert_eq!(test3.id(), ".foo[42].bar[1337]".to_string());
    }

    #[test]
    fn validation_error_double_index() {
        // XXX is this even legal? If it was it's at least supposed to be `.[42][1337]`?,
        // XXX verify with Python side.
        let test0 = ValidationError {
            message: String::new(),
            path: vec![ValidationPath::Index(42), ValidationPath::Index(1337)],
        };

        assert_eq!(test0.id(), "[42][1337]".to_string());
    }

    #[test]
    fn validation_result_no_error_valid() {
        let result = ValidationResult::new(String::new());
        let valid: bool = result.into();

        assert_eq!(valid, true);
    }

    #[test]
    fn validation_result_error_invalid() {
        let mut result = ValidationResult::new(String::new());
        result.fail("booboo".to_string());

        let valid: bool = result.into();

        assert_eq!(valid, false);
    }
}