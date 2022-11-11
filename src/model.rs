use onnxruntime;

pub mod use_onnxruntime {

    use super::onnxruntime::{
        environment::Environment, session::Session, GraphOptimizationLevel, LoggingLevel, OrtError,
    };

    pub fn get_environment(name: &str) -> Result<Environment, OrtError> {
        Environment::builder()
            .with_name(name)
            .with_log_level(LoggingLevel::Verbose)
            .build()
    }

    pub fn initialize_model<'env, 'a>(
        environment: &'env Environment,
        model_path: String,
        num_threads: i16,
    ) -> Result<Session<'env>, OrtError> {
        let model = environment
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_number_threads(num_threads)?
            .with_model_from_file(model_path)?;
        Ok(model)
    }
}
