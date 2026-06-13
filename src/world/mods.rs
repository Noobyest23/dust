use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDefinition {
    pub name: String,
    pub color: [u8; 4],
    pub density: i32,
    pub conductivity: f32,
    pub movement_type: String, 
    pub base_temp: f32,
    pub flammability: f32,
    pub corrosive_resistance: f32,
}

#[derive(Debug, Clone)]
pub struct ModLoader {
    pub elements: Vec<ElementDefinition>,
}

impl ModLoader {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    
    pub fn load_mods(mods_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut loader = Self::new();

        if !mods_path.exists() {
            println!("Mods folder not found at {:?}, creating it", mods_path);
            fs::create_dir_all(mods_path)?;
            return Ok(loader);
        }

        
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    match ext.to_str() {
                        Some("toml") => {
                            if let Err(e) = loader.load_toml_file(&path) {
                                eprintln!("Error loading TOML file {:?}: {}", path, e);
                            }
                        }
                        Some("json") => {
                            if let Err(e) = loader.load_json_file(&path) {
                                eprintln!("Error loading JSON file {:?}: {}", path, e);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        println!(
            "Loaded {} custom elements from mods folder",
            loader.elements.len()
        );
        Ok(loader)
    }

    fn load_toml_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let elem: ElementDefinition = toml::from_str(&content)?;
        println!("Loaded custom element from TOML: {}", elem.name);
        self.elements.push(elem);
        Ok(())
    }

    fn load_json_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let elem: ElementDefinition = serde_json::from_str(&content)?;
        println!("Loaded custom element from JSON: {}", elem.name);
        self.elements.push(elem);
        Ok(())
    }

    pub fn get_element(&self, index: usize) -> Option<&ElementDefinition> {
        self.elements.get(index)
    }

    pub fn list_elements(&self) -> Vec<&String> {
        self.elements.iter().map(|e| &e.name).collect()
    }
}


pub fn generate_example_mods(mods_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(mods_path)?;

    
    let example_toml = r#"name = "Copper Wire"
color = [184, 115, 51, 255]
density = 45
conductivity = 5.0
movement_type = "static"
base_temp = 20.0
flammability = 0.0
corrosive_resistance = 0.8
"#;

    
    let example_json = r#"{
  "name": "Liquid Metal",
  "color": [200, 100, 50, 255],
  "density": 60,
  "conductivity": 3.5,
  "movement_type": "liquid",
  "base_temp": 100.0,
  "flammability": 0.0,
  "corrosive_resistance": 0.7
}
"#;

    let toml_path = mods_path.join("example.toml");
    let json_path = mods_path.join("example.json");

    if !toml_path.exists() {
        fs::write(&toml_path, example_toml)?;
        println!("Created example TOML mod at {:?}", toml_path);
    }

    if !json_path.exists() {
        fs::write(&json_path, example_json)?;
        println!("Created example JSON mod at {:?}", json_path);
    }

    Ok(())
}
