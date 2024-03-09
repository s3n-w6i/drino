use std::fmt::Error;
use crate::dataset::Dataset;
use crate::preprocessing_steps::step2_import_data::{ImportStepExtra, ImportStepOutput};
use crate::preprocessing_steps::step3_validate_data::rule_severity::Severity;
use crate::preprocessing_steps::step3_validate_data::rule_violations::RuleViolations;
use crate::preprocessing_steps::step3_validate_data::rules::Rule;

pub(crate) mod rules;
pub(crate) mod rule_severity;
pub(crate) mod rule_violations;

pub async fn validate_data(
    imported_data: ImportStepOutput
) -> Result<ValidateStepOutput, ValidateError> {
    // TODO: Validate data
    Ok(ValidateStepOutput {
        dataset: imported_data.dataset,
        extra: imported_data.extra,
        skip: false // TODO: Decide based on validation errors
    })
}

async fn validate_gtfs() -> Result<(Dataset, ), Error> {
    /*match files_read_result {
        Ok(_) => {
            for rule in gtfs_rules() {
                let violations = rule.get_violations(&ctx).await.expect("Unable to verify rules");
                let count = violations.clone().count().await.expect("Unable to count violations");
                if count > 0 {
                    println!("{count} violations against rule '{:?}' (first 10 samples):", rule);
                    violations.show_limit(10).await.expect("Unable to display violations");
                }
            }
        }
        Err(error) => {
            match error {
                DataFusionError::SchemaError(error, ..) => {
                    println!("{}", error)
                }
                _ => panic!("Unknown error while reading files: {}", error),
            }
        }
    }*/

    Ok(todo!())
}

#[derive(thiserror::Error, Debug)]
pub enum ValidateError {
    //RuleViolations(Vec<Box<dyn RuleViolations/*<dyn Rule<dyn Severity>, dyn Severity>*/>>)
}

/*impl Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*let err: &dyn Display = match self {
            ValidateError::RuleViolations(violations) => violations
        };
        write!(f, "{}", err)*/
        write!(f, "")
    }
}*/

pub struct ValidateStepOutput {
    pub(crate) dataset: Dataset,
    pub(crate) extra: ImportStepExtra,
    pub(crate) skip: bool
}