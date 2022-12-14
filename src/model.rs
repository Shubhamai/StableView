#[allow(non_snake_case)]
pub mod OnnxSessionsManager {

    use onnxruntime::{
        environment::Environment, session::Session, GraphOptimizationLevel, LoggingLevel, OrtError,
    };

    // Setting up the environment 
    pub fn get_environment(name: &str) -> Result<Environment, OrtError> {
        Environment::builder()
            .with_name(name)
            .with_log_level(LoggingLevel::Verbose)
            .build()
    }

    // Setting up the session, logging levels, optimization levels and threads.
    pub fn initialize_model(
        environment: &Environment,
        model_path: String,
        num_threads: i16,
    ) -> Result<Session, OrtError> {
        let model = environment
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_number_threads(num_threads)?
            .with_model_from_file(model_path)?;
        Ok(model)
    }
}
