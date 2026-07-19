use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

pub type Element = u16;

#[derive(PartialEq, Clone, Debug, Deserialize, Copy)]
pub enum Movement {
    Static,  
    Powder,  
    Liquid,  
    Gas,     
}

#[derive(Clone, Deserialize)]
pub enum Condition {
    LifetimeGreater(u32),
    TemperatureAbove(f32),
    TemperatureBelow(f32),
    RandomChance(f32),
    NearElement(Element),
    NotNearElement(Element),
    IsInsideOf(Element),
    IsNotInsideOf(Element),
    NearTemperatureAbove(f32),
    NearTemperatureBelow(f32),
    IsElementInRadius(Element, u32),
    HasChargeAbove(f32),
    HasChargeBelow(f32),
    NearElementType(Movement),
}

#[derive(Clone, Deserialize)]
pub struct Reaction {
    pub conditions: Vec<Condition>,
    pub output: Vec<(Element, usize)>,
}

#[derive(Deserialize)]
pub struct ElemDefinition {
    pub name: String,
    pub id: u16,
    pub movement: Movement, 
    pub density: f32, 
    pub color: [u8; 4],
    pub hidden: bool, 
    pub super_hidden: bool, 
    pub thermal_conductivity: f32, 
    pub electrical_conductivity: f32, 
    pub corrosiveness: f32,
    pub flammability: f32,
    pub reactions: Vec<Reaction>,
}

impl ElemDefinition {
    /// Loads a single element definition from a given JSON file path
    pub fn load_from_json(path: &str) -> Option<ElemDefinition> {
        // Open the file safely
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        // Deserialize the JSON directly from the file buffer
        let definition: ElemDefinition = serde_json::from_reader(reader).ok()?;
        
        Some(definition)
    }

    /// Alternative: If you are loading an array of all elements from one file
    pub fn load_all_from_json(path: &str) -> Option<Vec<ElemDefinition>> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let definitions: Vec<ElemDefinition> = serde_json::from_reader(reader).ok()?;
        Some(definitions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_elements_success() {
        // Mocking data directly so the test doesn't rely on external file structures
        let json_data = r#"
        [
          {
            "name": "Air",
            "id": 0,
            "movement": "Gas",
            "density": 0.0012,
            "color": [0, 0, 0, 0],
            "hidden": false,
            "super_hidden": false,
            "thermal_conductivity": 0.02,
            "electrical_conductivity": 0.0,
            "corrosiveness": 0.0,
            "flammibility": 0.0,
            "reactions": []
          },
          {
            "name": "Stone",
            "id": 1,
            "movement": "Static",
            "density": 2.5,
            "color": [175, 175, 175, 255],  
            "hidden": false,
            "super_hidden": false,
            "thermal_conductivity": 0.3,
            "electrical_conductivity": 0.0,
            "corrosiveness": 0.0,
            "flammibility": 0.0,
            "reactions": [
              {
                "conditions": [{ "TemperatureAbove": 1200.0 }],
                "output": [[11, 1]]
              }
            ]
          }
        ]
        "#;

        // Parse from string slice instead of reader for the unit test
        let result: Result<Vec<ElemDefinition>, _> = serde_json::from_str(json_data);

        assert!(result.is_ok(), "Failed to parse valid JSON Array: {:?}", result.err());
        let list = result.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Air");
        assert_eq!(list[1].name, "Stone");
        assert_eq!(list[1].reactions.len(), 1);
    }
}