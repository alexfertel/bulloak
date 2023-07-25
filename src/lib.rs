use std::fs;

pub fn run(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tree = fs::read_to_string(file_name)?;

    println!("Tree: {}", tree);

    Ok(())
}
