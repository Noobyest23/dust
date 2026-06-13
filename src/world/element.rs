use strum::IntoEnumIterator;
use strum_macros::{EnumIter, Display, EnumCount, EnumString};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

use crate::world::World;

macro_rules! melting_point {
    ($temp:expr) => {
        Reaction {
            conditions: vec![
                Condition::TemperatureAbove($temp),
                Condition::RandomChance(0.05),
            ],
            output: vec![(Element::Lava, 1)],
        }
    };
}

#[derive(Clone, Copy, PartialEq, Debug, EnumIter, Display, Hash, Eq, EnumCount, EnumString)]
pub enum Element {
    Air,
    Stone,
    Water,
    Sand,
    Steam,
    Fire,
    Smoke,
    Ice,
    Copper,
    #[strum(serialize = "Oxidized Copper")]
    AgedCopper,
    Zinc,
    Lava,
    Obsidian,
    Wood,
    Oil,
    Ash,
    Charcoal,
    #[strum(serialize = "Liquid Nitrogen")]
    LiquidNitrogen,
    Cloud,
    #[strum(serialize = "Packed Sand")]
    PackedSand,
    Dirt,
    Iron,
    Acid,
    Dust,
    Plant,
    Lamp,
    #[strum(serialize = "Charged Lamp")]
    LampOn,
    Battery,
    Wire,
    Electricity,
    Rust, 
    Turbine,
    Glass,
    Steel,
    Brass,
    Hydrogen,
    #[strum(serialize = "Wet Concrete")]
    WetConcrete,
    Concrete,
    Fungus,
    Explosion,
    UserDefined(u32), 
    Gold,
    Snow,
    Nitrogen,
    Ammonia,
    Oxygen,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Movement {
    Static,  
    Powder,  
    Liquid,  
    Gas,     
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Reaction {
    pub conditions: Vec<Condition>,
    pub output: Vec<(Element, usize)>,
}

pub fn can_displace(source: Element, target: Element) -> bool {
    
    if source == Element::Air {
        return false;
    }

    let source_move = source.movement_type();
     
    let target_move = target.movement_type();
    if source_move == Movement::Static || target_move == Movement::Static {
        return false;
    }
    match source_move {
        
        
        Movement::Gas => {
            let is_target_solid = target_move == Movement::Static || target_move == Movement::Powder;
            !is_target_solid && source.density() < target.density()
        },
        
        _ => source.density() > target.density(),
    }
}


impl Element {

    pub fn to_u32(&self) -> u32 {
        match self {
            Element::Air => 0,
            Element::Stone => 1,
            Element::Water => 2,
            Element::Sand => 3,
            Element::Steam => 4,
            Element::Fire => 5,
            Element::Smoke => 6,
            Element::Ice => 7,
            Element::Copper => 8,
            Element::AgedCopper => 9,
            Element::Zinc => 10,
            Element::Lava => 11,
            Element::Obsidian => 12,
            Element::Wood => 13,
            Element::Oil => 14,
            Element::Ash => 15,
            Element::Charcoal => 16,
            Element::LiquidNitrogen => 17,
            Element::Cloud => 18,
            Element::PackedSand => 19,
            Element::Dirt => 20,
            Element::Iron => 21,
            Element::Acid => 22,
            Element::Dust => 23,
            Element::Plant => 24,
            Element::Lamp => 25,
            Element::LampOn => 26,
            Element::Battery => 27,
            Element::Wire => 28,
            Element::Electricity => 29,
            Element::Rust => 30,
            Element::Turbine => 31,
            Element::Glass => 32,
            Element::Steel => 33,
            Element::Brass => 34,
            Element::Hydrogen => 35,
            Element::WetConcrete => 36,
            Element::Concrete => 37,
            Element::Fungus => 38,
            Element::Explosion => 39,
            Element::UserDefined(_) => 40,
            Element::Gold => 41,
            Element::Snow => 42,
            Element::Nitrogen => 43,
            Element::Ammonia => 44,
            Element::Oxygen => 45,
        }
    }

    pub fn from_string(name: String) -> Option<Element> {
        
        Element::iter()
            .find(|elem| elem.to_string().eq_ignore_ascii_case(&name))
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Element::Smoke => true,
            Element::Steam => true,
            Element::AgedCopper => true,
            Element::Charcoal => true,
            Element::Ash => true,
            Element::Cloud => true,
            Element::PackedSand => true,
            Element::LampOn => true,
            Element::Electricity => true,
            Element::Rust => true,
            Element::Glass => true,
            Element::Steel => true,
            Element::Brass => true,
            Element::Hydrogen => true,
            Element::Concrete => true,
            Element::WetConcrete => true,
            Element::Obsidian => true,
            Element::Plant => true,
            Element::Fungus => true,
            Element::Explosion => true,
            Element::UserDefined(_) => true,
            Element::Snow => true,
            Element::LiquidNitrogen => true,
            Element::Ammonia => true,
            Element::Oxygen => true,
            _ => false,
        }
    }

    pub fn is_super_hidden(&self) -> bool {
        match self {
            Element::LampOn => true,
            Element::Electricity => true,
            Element::Explosion => true,
            Element::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn movement_type(&self) -> Movement {
        match self {
            Element::Stone => Movement::Static,
            Element::Sand => Movement::Powder,
            Element::Water => Movement::Liquid,
            Element::Air => Movement::Gas,
            Element::Fire => Movement::Gas,   
            Element::Smoke | Element::Steam => Movement::Gas,
            Element::Ice => Movement::Static,
            Element::Copper => Movement::Static,
            Element::AgedCopper => Movement::Static,
            Element::Lava => Movement::Liquid,
            Element::Obsidian => Movement::Static,
            Element::Wood => Movement::Static,
            Element::Oil => Movement::Liquid,
            Element::Ash => Movement::Powder,
            Element::Charcoal => Movement::Static,
            Element::PackedSand => Movement::Static,
            Element::LiquidNitrogen => Movement::Liquid,
            Element::Cloud => Movement::Gas,
            Element::Dirt => Movement::Static,
            Element::Iron => Movement::Static,
            Element::Acid => Movement::Liquid,
            Element::Dust => Movement::Powder,
            Element::Plant => Movement::Static,
            Element::Lamp => Movement::Static,
            Element::LampOn => Movement::Static,
            Element::Battery => Movement::Static,
            Element::Wire => Movement::Static,
            Element::Electricity => Movement::Static,
            Element::Zinc => Movement::Static,
            Element::Rust => Movement::Powder,
            Element::Turbine => Movement::Static,
            Element::Glass => Movement::Static,
            Element::Steel => Movement::Static,
            Element::Brass => Movement::Static,
            Element::Hydrogen => Movement::Gas,
            Element::Concrete => Movement::Static,
            Element::WetConcrete => Movement::Liquid,
            Element::Fungus => Movement::Static,
            Element::Explosion => Movement::Static,
            Element::UserDefined(_) => Movement::Static, 
            Element::Gold => Movement::Static,
            Element::Snow => Movement::Powder,
            Element::Nitrogen => Movement::Gas,
            Element::Ammonia => Movement::Gas,
            Element::Oxygen => Movement::Gas,
        }
    }

    pub fn density(&self) -> i32 {
    match self {
        Element::Stone => 100,
        Element::Sand  => 50,
        Element::Water => 10,
        Element::Air   => 5,
        Element::Fire  => 4,  
        Element::Smoke => 2,  
        Element::Steam => 1,  
        Element::Ice => 80,
        Element::Copper => 100,
        Element::AgedCopper => 100,
        Element::Lava => 20,
        Element::Obsidian => 300,
        Element::Wood => 70,
        Element::Oil => 8,
        Element::Ash => 30,
        Element::Charcoal => 100,
        Element::LiquidNitrogen => 15,
        Element::Cloud => 1,
        Element::PackedSand => 70,
        Element::Dirt => 60,
        Element::Iron => 200,
        Element::Acid => 9,
        Element::Dust => 40,
        Element::Plant => 20,
        Element::Lamp => 50,
        Element::LampOn => 50,
        Element::Battery => 75,
        Element::Wire => 10,
        Element::Electricity => 10,
        Element::Zinc => 100,
        Element::Rust => 180,
        Element::Turbine => 100,
        Element::Glass => 60,
        Element::Steel => 200,
        Element::Brass => 200,
        Element::Hydrogen => 0,
        Element::WetConcrete => 15,
        Element::Concrete => 300,
        Element::Fungus => 30,
        Element::Explosion => 10,
        Element::UserDefined(_) => 100, 
        Element::Gold => 100,
        Element::Snow => 42,
        Element::Nitrogen => 0,
        Element::Ammonia => 0,
        Element::Oxygen => 0,
    }
}

    pub fn conductivity(&self) -> f32 {
    match self {
        Element::Copper => 2.0,      
        Element::AgedCopper => 0.9,  
        Element::Water => 0.76,       
        Element::Lava => 0.5,        
        Element::Steam => 0.6,       
        Element::Fire => 0.8,        
        Element::Ice => 0.3,         
        Element::Stone => 0.15,      
        Element::Obsidian => 0.12,   
        Element::Sand => 0.08,       
        Element::Air => 0.05,        
        Element::Smoke => 0.04,      
        Element::Wood => 0.04,
        Element::Oil => 0.5,
        Element::Ash => 0.12,
        Element::Charcoal => 0.04,
        Element::LiquidNitrogen => 0.5,
        Element::Cloud => 0.02,
        Element::PackedSand => 0.1,
        Element::Dirt => 0.1,
        Element::Iron => 0.17,
        Element::Acid => 0.6,
        Element::Dust => 0.05,
        Element::Plant => 0.1,
        Element::Lamp => 0.99,
        Element::LampOn => 0.99,
        Element::Battery => 1.0,
        Element::Wire => 10.0,
        Element::Electricity => 0.0,
        Element::Zinc => 0.29,
        Element::Rust => 0.14,
        Element::Turbine => 0.17,
        Element::Steel => 0.15,      
        Element::Brass => 0.46,
        Element::Glass => 0.01,      
        Element::Hydrogen => 0.02,   
        Element::Concrete => 0.05,   
        Element::WetConcrete => 0.4, 
        Element::Fungus => 0.5,
        Element::Explosion => 0.5,
        Element::UserDefined(_) => 0.1, 
        Element::Gold => 2.5,
        Element::Snow => 0.1,
        Element::Nitrogen => 0.01,
        Element::Ammonia => 0.01,
        Element::Oxygen => 0.01,
    }
}

    pub fn color(&self) -> [u8; 4] {
        match self {
            Element::Air => [30u8, 30u8, 30u8, 255u8],
            Element::Sand => [230u8, 190u8, 100u8, 255u8],
            Element::Water => [50u8, 100u8, 255u8, 255u8],
            Element::Stone => [120u8, 120u8, 120u8, 255u8],
            Element::Fire => [255, 100, 0, 255],
            Element::Smoke => [60, 60, 60, 255],
            Element::Steam => [200, 200, 255, 180],
            Element::Ice => [180, 220, 255, 180],
            Element::Copper => [163, 72, 39, 255],
            Element::AgedCopper => [80, 140, 110, 255],
            Element::Lava => [255, 60, 0, 255],
            Element::Obsidian => [15, 0, 15, 255],
            Element::Wood => [114, 92, 66, 255],
            Element::Oil => [5, 5, 5, 255],
            Element::Ash => [125, 125, 125, 255],
            Element::Charcoal => [1, 1, 1, 255],
            Element::LiquidNitrogen => [200, 255, 255, 255],
            Element::Cloud => [220, 220, 220, 200],
            Element::PackedSand => [210, 180, 140, 255],
            Element::Dirt => [100, 60, 20, 255],
            Element::Iron => [100, 100, 100, 255],
            Element::Acid => [50, 200, 50, 255],
            Element::Dust => [150, 150, 150, 255],
            Element::Plant => [20, 150, 20, 255],
            Element::Lamp => [20, 20, 20, 255],
            Element::LampOn => [255, 255, 150, 255],
            Element::Battery => [181, 148, 16, 255],
            Element::Wire => [128, 0, 0, 255],
            Element::Electricity => [255, 255, 0, 255],
            Element::Zinc => [175, 185, 200, 255],      
            Element::Steel => [110, 120, 130, 255],     
            Element::Glass => [180, 240, 255, 100],     
            Element::Hydrogen => [255, 200, 255, 50],   
            Element::Concrete => [150, 150, 140, 255],  
            Element::Rust => [140, 40, 40, 255],
            Element::Turbine => [200, 200, 200, 255],
            Element::Brass => [225, 190, 60, 255],
            Element::WetConcrete => [80, 85, 90, 255],
            Element::Fungus => [180, 156, 156, 255],
            Element::Explosion => [182, 92, 42, 255],
            Element::UserDefined(_) => [255, 0, 255, 255],
            Element::Gold => [255, 215, 0, 255],
            Element::Snow => [240, 240, 255, 255],
            Element::Nitrogen => [255, 255, 255, 80],
            Element::Ammonia => [200, 255, 255, 255],
            Element::Oxygen => [200, 255, 255, 255],
        }
    }

    pub fn base_temp(&self) -> f32 {
        match self {
            Element::Fire => 300.0,
            Element::Lava => 500.0,
            Element::Ice => -35.0,
            Element::Steam => 120.0,
            Element::Ash => 150.0,
            Element::LiquidNitrogen => -196.0,
            Element::Cloud => 0.0, 
            Element::Gold => 100.0,
            Element::Snow => -10.0,
            _ => 20.0 
        }
    }


    pub fn corrosive_resistance(&self) -> f32 {
        match self {
            
            Element::Obsidian | Element::Lava | Element::Air | Element::Electricity | Element::Smoke => 1.0,
            Element::Acid => 1.0,      
            Element::Glass => 1.0,     
            Element::Iron => 1.0,      
            Element::Steel => 0.95,    
            Element::Concrete => 0.9,  
            Element::Sand | Element::PackedSand | Element::Rust => 0.9,

            
            Element::Stone | Element::Battery => 0.8,
            Element::Brass => 0.7,     
            Element::AgedCopper => 0.8, 

            
            Element::Copper => 0.6,    
            Element::Water => 0.5,
            Element::WetConcrete => 0.4, 
            Element::Zinc => 0.3,      
            
            
            Element::Hydrogen => 1.0,  
            Element::Wood | Element::Plant => 0.2, 
            Element::Gold => 0.9,
            _ => 0.0,
        }
    }

    pub fn is_corrosive(&self) -> bool {
        match self {
            Element::Acid => true,
            _ => false,
        }
    }

    pub fn flammability(&self) -> f32 {
        match self {
            Element::Dust => 1.0, 
            Element::Wood => 0.8, 
            Element::Oil => 0.9,  
            Element::Ash => 0.2,  
            Element::Charcoal => 0.75, 
            Element::Plant => 0.9,
            Element::Fungus => 0.9,
            Element::Hydrogen => 1.0,
            Element::Explosion => 1.0,
            Element::Ammonia => 0.5,
            Element::Oxygen => 0.5,
            _ => 0.0,
        }
    }

    pub fn growth_range(&self) -> Option<(f32, f32)> {
        match self {
            Element::Plant => Some((15.0, 30.0)), 
            Element::Fungus => Some((25.0, 30.0)),
            _ => None,
        }
    }

    pub fn source_temp(&self) -> Option<f32> {
        
            
            
            
        
        
        None
    }

    
    pub fn get_reactions(&self) -> Vec<Reaction> {
        match self {
            Element::Water => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Steam, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureBelow(0.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Ice, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(5000),
                        Condition::RandomChance(0.0001),
                        Condition::IsNotInsideOf(Element::Water),
                    ],
                    output: vec![(Element::Cloud, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Plant),
                        Condition::RandomChance(0.01),
                    ],
                    output: vec![(Element::Plant, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::HasChargeAbove(1.0),
                        Condition::RandomChance(0.01),
                    ],
                    output: vec![
                        (Element::Oxygen, 1),
                        (Element::Hydrogen, 1),
                        (Element::Hydrogen, 1),
                    ],
                }
            ],
            Element::Steam => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureBelow(100.0),
                        Condition::RandomChance(0.002),
                    ],
                    output: vec![(Element::Water, 1)],
                }
            ],
            Element::Fire => vec![
                Reaction {
                    conditions: vec![
                        Condition::RandomChance(0.01),
                    ],
                    output: vec![(Element::Smoke, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::RandomChance(0.03),
                    ],
                    output: vec![(Element::Air, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Water),
                    ],
                    output: vec![(Element::Smoke, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureBelow(10.0),
                        Condition::RandomChance(0.3),
                    ],
                    output: vec![(Element::Smoke, 1)],
                },
            ],
            Element::Ice => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearTemperatureAbove(0.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Water, 1)],
                }
            ],
            Element::Copper => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Water),
                        Condition::RandomChance(0.002),
                        Condition::LifetimeGreater(2000),
                    ],
                    output: vec![(Element::AgedCopper, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::AgedCopper),
                        Condition::RandomChance(0.002),
                        Condition::LifetimeGreater(2000),
                    ],
                    output: vec![(Element::AgedCopper, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(20000),
                        Condition::RandomChance(0.0002),
                    ],
                    output: vec![(Element::AgedCopper, 1)],
                },
                melting_point!(1085.0)
            ],
            Element::AgedCopper => vec![
                melting_point!(1085.0),
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Ammonia),
                        Condition::RandomChance(0.01),
                    ],
                    output: vec![(Element::Copper, 1)],
                }
            ],
            Element::Lava => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearTemperatureBelow(50.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Obsidian, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Water),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Obsidian, 1)],
                },
            ],
            Element::Wood => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Fire, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Ash, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Charcoal, 1)],
                },
            ],
            Element::Oil => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Fire),
                        Condition::RandomChance(0.0008),
                    ],
                    output: vec![(Element::Fire, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(150.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Fire, 1)],
                }
            ],
            Element::Stone => vec![
                melting_point!(1200.0)
            ],
            Element::Smoke => vec![
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(100),
                        Condition::TemperatureBelow(50.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Air, 1)],
                }
            ],
            Element::Cloud => vec![
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(5000),
                        Condition::RandomChance(0.01),
                        Condition::TemperatureAbove(10.0)
                    ],
                    output: vec![(Element::Water, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(5000),
                        Condition::RandomChance(0.01),
                        Condition::TemperatureBelow(10.0)
                    ],
                    output: vec![(Element::Snow, 1)],
                }
            ],
            Element::Sand => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Water),
                        Condition::NearElement(Element::Dust), 
                        Condition::RandomChance(0.05),
                    ],
                    output: vec![(Element::WetConcrete, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::IsNotInsideOf(Element::Sand),
                        Condition::IsElementInRadius(Element::Water, 4),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::PackedSand, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(1000.0), 
                        Condition::RandomChance(0.1),
                    ],
                    output: vec![(Element::Glass, 1)],
                },
                
            ],
            Element::WetConcrete => vec![
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(1000), 
                        Condition::RandomChance(0.01),
                    ],
                    output: vec![(Element::Concrete, 1)],
                },
            ],
            Element::PackedSand => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(200.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Sand, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Water),
                        Condition::NearElement(Element::Dust), 
                        Condition::RandomChance(0.05),
                    ],
                    output: vec![(Element::WetConcrete, 1)],
                },
            ],
            Element::Dust => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(60.0),
                    ],
                    output: vec![(Element::Explosion, 2), (Element::Fire, 1)],
                }
            ],
            Element::Iron => vec![
                melting_point!(1538.0),
                Reaction {
                    conditions: vec![
                        Condition::IsNotInsideOf(Element::Iron),
                        Condition::IsElementInRadius(Element::Water, 4),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Rust, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Acid),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Hydrogen, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(20000),
                        Condition::RandomChance(0.0002),
                    ],
                    output: vec![(Element::Rust, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Charcoal),
                        Condition::TemperatureAbove(600.0),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Steel, 1)],
                },
            ],
            Element::Plant => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Fire, 1)],
                },
                
            ],
            Element::Lamp => vec![
                Reaction {
                    conditions: vec![
                        Condition::HasChargeAbove(0.2),
                    ],
                    output: vec![(Element::LampOn, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(800.0),
                    ],
                    output: vec![(Element::Lava, 1)],
                }
            ],
            Element::LampOn => vec![
                Reaction {
                    conditions: vec![
                        Condition::HasChargeBelow(0.2),
                    ],
                    output: vec![(Element::Lamp, 1)],
                }
            ],
            Element::Charcoal => vec![
                 
            ],
            Element::Electricity => vec![
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(2),
                    ],
                    output: vec![(Element::Air, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::RandomChance(0.2),
                    ],
                    output: vec![(Element::Electricity, 1)],
                }
            ],
            Element::Acid => vec![
                Reaction {
                    conditions: vec![
                        Condition::IsElementInRadius(Element::Copper, 3),
                        Condition::NearElement(Element::Zinc),
                    ],
                    output: vec![(Element::Electricity, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::IsElementInRadius(Element::AgedCopper, 3),
                        Condition::NearElement(Element::Zinc),
                    ],
                    output: vec![(Element::Electricity, 1)],
                }
            ],
            Element::Air => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElementType(Movement::Liquid),
                        Condition::NearElement(Element::Turbine),
                    ],
                    output: vec![(Element::Electricity, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElementType(Movement::Gas),
                        Condition::NearElement(Element::Turbine),
                    ],
                    output: vec![(Element::Electricity, 1)],
                }
            ],
            Element::Zinc => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Copper),
                        Condition::TemperatureAbove(400.0), 
                        Condition::RandomChance(0.05),
                    ],
                    output: vec![(Element::Brass, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Acid),
                        Condition::RandomChance(0.03),
                    ],
                    output: vec![(Element::Hydrogen, 32)],
                },
                melting_point!(619.0)
            ],
            Element::Hydrogen => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Fire),
                        Condition::RandomChance(0.5),
                    ],
                    output: vec![(Element::Explosion, 4)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(400.0),
                        Condition::RandomChance(0.8),
                        Condition::NearElement(Element::Nitrogen),
                    ],
                    output: vec![(Element::Ammonia, 1)],
                }
            ],
            Element::Concrete => vec![
                melting_point!(1500.0)
            ],
            Element::Brass => vec![
                melting_point!(930.0)
            ],
            Element::Glass => vec![
                melting_point!(1500.0)
            ],
            Element::Steel => vec![
                melting_point!(1370.0)
            ],
            Element::Dirt => vec![
                Reaction {
                    conditions: vec![
                        Condition::IsNotInsideOf(Element::Dirt),
                        Condition::TemperatureAbove(Element::Plant.growth_range().unwrap().0),
                        Condition::TemperatureBelow(Element::Plant.growth_range().unwrap().1),
                        Condition::IsElementInRadius(Element::Water, 4),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Plant, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::IsNotInsideOf(Element::Dirt),
                        Condition::TemperatureAbove(Element::Fungus.growth_range().unwrap().0),
                        Condition::TemperatureBelow(Element::Fungus.growth_range().unwrap().1),
                        Condition::IsElementInRadius(Element::Water, 4),
                        Condition::RandomChance(0.02),
                    ],
                    output: vec![(Element::Fungus, 1)],
                }
            ],
            Element::Fungus => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(100.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Fire, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Dirt),
                        Condition::RandomChance(0.002),
                    ],
                    output: vec![(Element::Fungus, 1)],
                }
            ],
            Element::Explosion => vec![
                Reaction {
                    conditions: vec![
                        Condition::RandomChance(0.3),
                    ],
                    output: vec![(Element::Fire, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::LifetimeGreater(10),
                        Condition::RandomChance(0.5),
                    ],
                    output: vec![(Element::Air, 1)],
                },
            ],
            Element::Gold => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(1064.0),
                    ],
                    output: vec![(Element::Lava, 1)],
                }
            ],
            Element::Snow => vec![
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(10.0),
                        Condition::RandomChance(0.008),
                    ],
                    output: vec![(Element::Water, 1)],
                }
            ],
            Element::Nitrogen => vec![
                
                Reaction {
                    conditions: vec![
                        Condition::TemperatureBelow(-196.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::LiquidNitrogen, 1)],
                },
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(400.0),
                        Condition::RandomChance(0.8),
                        Condition::NearElement(Element::Hydrogen),
                    ],
                    output: vec![(Element::Ammonia, 1)],
                }
            ],
            Element::LiquidNitrogen => vec![
                
                Reaction {
                    conditions: vec![
                        Condition::TemperatureAbove(-196.0),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Nitrogen, 1)],
                },
            ],
            Element::Rust => vec![
                Reaction {
                    conditions: vec![
                        Condition::NearElement(Element::Ammonia),
                        Condition::RandomChance(0.08),
                    ],
                    output: vec![(Element::Iron, 1)],
                },
            ],
            _ => vec![]
        }
    }

}