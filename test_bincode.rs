// Quick test to see if CodeGraph can be serialized with bincode
use code_navigator::core::CodeGraph;

fn main() {
    let mut graph = CodeGraph::new("/test".to_string(), "typescript".to_string());

    println!("Testing bincode serialization...");

    // Try to serialize
    match bincode::serialize(&graph) {
        Ok(data) => {
            println!("✓ Serialization successful: {} bytes", data.len());

            // Try to deserialize
            match bincode::deserialize::<CodeGraph>(&data) {
                Ok(_) => println!("✓ Deserialization successful!"),
                Err(e) => println!("✗ Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("✗ Serialization failed: {}", e),
    }
}
