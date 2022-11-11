use onnxruntime::{
    environment::Environment, ndarray::Array, tensor::OrtOwnedTensor, GraphOptimizationLevel,
};

pub mod use_onnxruntime {

    pub fn initialize_model(
        file_path: String,
        num_threads: u32,
    ) -> Result<Environment, Box<dyn std::error::Error>> {

        let environment = Environment::builder().build()?;

        environment
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            .with_number_threads(num_threads)?
            .with_model_from_file(file_path)?
    }
}
