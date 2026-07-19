pub mod element;
pub mod mods;
use mods::ModLoader;
use std::collections::{HashMap, HashSet};
use std::io::{Write};

#[cfg(target_arch = "wasm32")]
use web_sys::{Storage, Window};

use crate::world::element::{Condition, ElemDefinition, Movement};
use element::{Element, Reaction};

pub struct World {
    pub width: usize,
    pub height: usize,
    pub elements: Vec<Element>,
    pub shades: Vec<u8>,
    pub velocities: Vec<i8>,
    pub temperatures: Vec<f32>,
    pub last_update: Vec<u32>,
    pub frame_count: u32,
    pub lifetimes: Vec<u32>,
    pub ambient_temp: f32,
    pub reaction_lookup: Vec<Vec<Reaction>>,
    pub time_effects: Vec<f32>,
    pub electrical_charge: Vec<f32>,
    pub force: Vec<(f32, f32)>,
    pub last_expensive_check: Vec<u32>,
    
    pub discovered_elements: HashSet<Element>,
    pub new_discovery: Option<Element>,
    pub process: bool,
    pub definitions: HashMap<u16, ElemDefinition>,
}

impl World {
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    pub fn get(&self, x: usize, y: usize) -> Element {
        
        if x >= self.width || y >= self.height {
            return 1;
        }
        self.elements[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, element: Element) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.elements[idx] = element;
            self.last_update[idx] = self.frame_count;
            self.last_expensive_check[idx] = self.frame_count;

            self.shades[idx] = if element == 0 {
                0
            } else {
                rand::random::<u8>()
            };
            self.velocities[idx] = if rand::random() { 1 } else { -1 };
            self.lifetimes[idx] = 0;

            if !self.discovered_elements.contains(&element) && !self.definitions.get(&element).unwrap().super_hidden {
                self.discovered_elements.insert(element);
                println!("New element discovered: {:?}", element);
                self.new_discovery = Some(element);
                let _ = self.save_discoveries("discovered_elements.txt");
            }

        }
    }

    pub fn add_heat(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let idx = self.get_index(x, y);
            self.temperatures[idx] += amount;
            
            self.last_update[idx] = self.frame_count;
        }
    }

    pub fn add_cold(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let idx = self.get_index(x, y);
            self.temperatures[idx] -= amount;
            
            self.last_update[idx] = self.frame_count;
        }
    }

    pub fn get_shade(&self, x: usize, y: usize) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.shades[y * self.width + x]
    }

    pub fn swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        
        if x1 < self.width && y1 < self.height && x2 < self.width && y2 < self.height {
            let idx1 = self.get_index(x1, y1);
            let idx2 = self.get_index(x2, y2);

            self.elements.swap(idx1, idx2);
            self.shades.swap(idx1, idx2);
            self.velocities.swap(idx1, idx2);
            self.temperatures.swap(idx1, idx2);
            self.lifetimes.swap(idx1, idx2);
            self.last_expensive_check.swap(idx1, idx2);

            
            
            self.last_update[idx1] = self.frame_count;
            self.last_update[idx2] = self.frame_count;
        }
    }

    fn try_swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, current: Element) -> bool {
        if x2 >= self.width || y2 >= self.height {
            return false;
        }

        let target = self.get(x2, y2);
        if self.can_displace(&current, &target) {
            self.swap(x1, y1, x2, y2);
            return true;
        }

        false
    }
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let mut discovered_elements = HashSet::new();
        let mut definitions = HashMap::new();

        let vec_definitions = ElemDefinition::load_all_from_json("dust.json")
            .expect("Unable to load dust.json");

        // Dynamically find the maximum element ID to safely size our reaction lookup vector
        let max_id = vec_definitions.iter().map(|d| d.id).max().unwrap_or(0) as usize;
        let mut reaction_lookup = vec![Vec::new(); max_id + 1];

        for elem_def in vec_definitions {
            let id = elem_def.id;

            // Map the element's reactions directly to its ID index for rapid O(1) checks during simulation updates
            reaction_lookup[id as usize] = elem_def.reactions.clone();
            
            if !elem_def.hidden && !elem_def.super_hidden {
                discovered_elements.insert(id);
            }

            definitions.insert(id, elem_def);
        }
        
        let mod_loader = match ModLoader::load_mods(std::path::Path::new("mods")) {
            Ok(loader) => loader,
            Err(e) => {
                eprintln!("Failed to load mods: {}", e);
                ModLoader::new()
            }
        };
        
        Self {
            width,
            height,
            elements: vec![0; width * height],
            shades: vec![0; width * height],
            velocities: vec![0; width * height],
            temperatures: vec![20.0; width * height],
            last_update: vec![0; width * height],
            frame_count: 0,
            lifetimes: vec![0; width * height],
            ambient_temp: 20.0,
            reaction_lookup,
            time_effects: vec![0.0; width * height],
            electrical_charge: vec![0.0; width * height],
            force: vec![(0.0, 0.0); width * height],
            last_expensive_check: vec![0; width * height],
            discovered_elements,
            new_discovery: None,
            process: true,
            definitions,
        }
    }
}


impl World {
    pub fn update(&mut self) {
        if !self.process {
            self.process = true;
            self.update_heat();
            self.update_physics();
            self.update_time_field();
            self.update_electrical_field();
            
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = self.get_index(x, y);
                    if self.elements[idx] == 0 && self.temperatures[idx] == self.ambient_temp
                    {
                        continue; 
                    }
                    self.process_reactions(x, y);
                }
            }
            return;
        }
        self.process = false;

        self.frame_count += 1;
        

        
        
        for y in 0..self.height {
            let left_to_right = rand::random::<bool>();

            
            let mut x_range: Vec<usize> = (0..self.width).collect();
            if !left_to_right {
                x_range.reverse();
            }

            for x in x_range {
                let current = self.get(x, y);
                let index = self.get_index(x, y);
                if self.last_update[index] >= self.frame_count {
                    continue; 
                }

                
                
                
                let local_time = 1.0 + self.time_effects[index];
                if local_time < 1.0 && rand::random::<f32>() > local_time {
                    continue;
                }

                
                if current == 0 {
                    continue;
                }

                self.process_reactions(x, y);

                if self.definitions.get(&current).unwrap().movement == Movement::Gas {
                    self.move_gas(x, y, current);
                }
            }
        }

        
        for y in (0..self.height).rev() {
            let left_to_right = rand::random::<bool>();

            
            let mut x_range: Vec<usize> = (0..self.width).collect();
            if !left_to_right {
                x_range.reverse();
            }

            for x in x_range {
                let idx = self.get_index(x, y);

                
                
                
                let local_time = 1.0 + self.time_effects[idx];
                if local_time < 1.0 && rand::random::<f32>() > local_time {
                    continue;
                }

                
                
                if self.last_update[idx] == self.frame_count {
                    continue;
                }

                let current = self.get(x, y);
                let move_type = self.definitions.get(&current).unwrap().movement;

                if move_type == Movement::Powder || move_type == Movement::Liquid {
                    let order = if rand::random() {
                        [-1isize, 1isize]
                    } else {
                        [1isize, -1isize]
                    };

                    if move_type == Movement::Powder {
                        self.move_sand(x, y, current, order);
                    } else {
                        self.move_water(x, y, current, order);
                    }
                }
            }
        }
    }

    fn update_physics(&mut self) {
        self.update_force();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let current_elem = self.elements[idx];
                let local_time = 1.0 + self.time_effects[idx];

                if current_elem == 29 {
                    self.electrical_charge[idx] = 100.0;
                }

                if current_elem != 0 {
                    self.lifetimes[idx] += local_time.max(0.0) as u32;
                    
                    

                    
                    /*if let Some((min_temp, max_temp)) = current_elem.growth_range() { 
                        let temp = self.temperatures[idx];
                        
                        
                        
                        if temp >= min_temp && temp <= max_temp && rand::random::<f32>() < (0.0005 * local_time) {
                            
                            
                            let neighbors = [
                                (x as i32 - 1, y as i32), (x as i32 + 1, y as i32),
                                (x as i32, y as i32 - 1), (x as i32, y as i32 + 1),
                            ];

                            for (nx, ny) in neighbors {
                                if self.is_in_bounds(nx, ny) {
                                    let n_idx = self.get_index(nx as usize, ny as usize);
                                    
                                    
                                    if self.elements[n_idx] == 0 {
                                        
                                        
                                        if self.is_touching_solid(nx as usize, ny as usize) {
                                            growth_actions.push((nx as usize, ny as usize, current_elem));
                                            break; 
                                        }
                                    }
                                }
                            }
                        }
                    } */
                }
            }
        }
    }

    pub fn update_heat(&mut self) {
        let ambient = self.ambient_temp;
        let width = self.width;
        let height = self.height;
        
        
        for idx in 0..width * height {
            let current_elem = self.elements[idx];
            
            let current_temp = self.temperatures[idx];
            let local_time = 1.0 + self.time_effects[idx];
            
            
            let base_rate = if current_elem == 0 { 0.8 } else { 0.01 };
            let mut cooling_rate = (base_rate * local_time).clamp(0.0, 0.5);
            let temp_diff = ambient - current_temp;
            if !temp_diff.is_nan() {
                
                
                
                let diff_mag = temp_diff.abs();
                let slowdown = (1.0 + (diff_mag / 50.0)).clamp(1.0, 20.0);
                cooling_rate /= slowdown;
                self.temperatures[idx] += temp_diff * cooling_rate;
            }
        }
        
        
        {
            let mut new_temps = self.temperatures.clone();
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;

                    let current_temp = self.temperatures[idx];
                    let local_time = 1.0 + self.time_effects[idx];

                    
                    let mut sum: f32 = 0.0;
                    let mut weight: f32 = 0.0;

                    if y > 0 {
                        let up_idx = (y - 1) * width + x;
                        sum += self.temperatures[up_idx] * 1.6f32;
                        weight += 1.6f32;
                    }
                    if y + 1 < height {
                        let down_idx = (y + 1) * width + x;
                        sum += self.temperatures[down_idx] * 1.0f32;
                        weight += 1.0f32;
                    }
                    if x > 0 {
                        let left_idx = y * width + (x - 1);
                        sum += self.temperatures[left_idx] * 1.0f32;
                        weight += 1.0f32;
                    }
                    if x + 1 < width {
                        let right_idx = y * width + (x + 1);
                        sum += self.temperatures[right_idx] * 1.0f32;
                        weight += 1.0f32;
                    }

                    if weight > 0.0 {
                        let neighbor_avg = sum / weight;
                        
                        let base_diffusion = 0.08f32;
                        let diffusion_rate = (base_diffusion * local_time).clamp(0.0f32, 0.6f32);
                        let delta = (neighbor_avg - current_temp) * diffusion_rate;
                        if !delta.is_nan() {
                            new_temps[idx] += delta;
                        }
                    }
                }
            }
            self.temperatures = new_temps;
        }
    }

    fn equalize_thermal_network(
        &mut self,
        start_x: usize,
        start_y: usize,
        visited: &mut Vec<bool>,
    ) {
        let target_elem = self.get(start_x, start_y);
        let mut group_indices = Vec::new();
        let mut total_temp = 0.0;

        
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_x, start_y));
        visited[self.get_index(start_x, start_y)] = true;

        while let Some((cx, cy)) = queue.pop_front() {
            let idx = self.get_index(cx, cy);
            group_indices.push(idx);
            total_temp += self.temperatures[idx];

            
            for (nx, ny) in [
                (cx as i32 - 1, cy as i32),
                (cx as i32 + 1, cy as i32),
                (cx as i32, cy as i32 - 1),
                (cx as i32, cy as i32 + 1),
            ] {
                if self.is_in_bounds(nx, ny) {
                    let n_idx = self.get_index(nx as usize, ny as usize);
                    if !visited[n_idx] && self.elements[n_idx] == target_elem {
                        visited[n_idx] = true;
                        queue.push_back((nx as usize, ny as usize));
                    }
                }
            }
        }

        
        if !group_indices.is_empty() {
            let avg_temp = total_temp / group_indices.len() as f32;
            for idx in group_indices {
                self.temperatures[idx] = avg_temp;
                
                self.last_update[idx] = self.frame_count;
            }
        }
    }

    pub fn update_time_field(&mut self) {
        
        let mut new_time = self.time_effects.clone();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                new_time[idx] *= 0.995;
                
                
                if x > 0 && x < self.width - 1 {
                    let left = y * self.width + (x - 1);
                    let right = y * self.width + (x + 1);
                    let avg = (self.time_effects[left] + self.time_effects[right]) * 0.5;
                    new_time[idx] = new_time[idx] * 0.8 + avg * 0.2;
                }
            }
        }
        
        self.time_effects = new_time;
    }

    pub fn update_electrical_field(&mut self) {
        
        
        let old_charge = self.electrical_charge.clone();
        let mut new_charge = old_charge.clone();
        let mut visited = vec![false; (self.width * self.height) as usize];

        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let elem_id = self.elements[idx];
                let elem = self.definitions.get(&elem_id).unwrap();
                if elem.electrical_conductivity > 1.0 && !visited[idx] {
                    let mut cluster = Vec::new();
                    let mut stack = vec![(x, y)];
                    visited[idx] = true;
                    let mut total_charge = 0.0;

                    while let Some((cx, cy)) = stack.pop() {
                        let c_idx = self.get_index(cx, cy);
                        cluster.push(c_idx);
                        total_charge += old_charge[c_idx];

                        for (nx, ny) in [(cx as i32 - 1, cy as i32), (cx as i32 + 1, cy as i32), (cx as i32, cy as i32 - 1), (cx as i32, cy as i32 + 1)] {
                            if self.is_in_bounds(nx, ny) {
                                let n_idx = self.get_index(nx as usize, ny as usize);
                                
                                if !visited[n_idx] && elem.electrical_conductivity > 1.0 {
                                    visited[n_idx] = true;
                                    stack.push((nx as usize, ny as usize));
                                }
                            }
                        }
                    }

                    if !cluster.is_empty() {
                        let average_charge = (total_charge / cluster.len() as f32) * 0.99;
                        for &pixel_idx in &cluster {
                            new_charge[pixel_idx] = average_charge;
                        }
                    }
                }

                if new_charge[idx] < 0.1 {
                    new_charge[idx] = 0.0;
                }
            }
        }

        
        for idx in 0..self.width * self.height {
            let current_charge = old_charge[idx];
            if current_charge <= 0.1 { continue; }

            let current_elem = self.elements[idx];
            let current_def = self.definitions.get(&current_elem).unwrap();
            let current_conduction = current_def.electrical_conductivity;
            let is_super_conductor = current_conduction >= 1.0;
            let is_battery = current_elem == 27;

            if !is_super_conductor && !is_battery {
                let resistance = (1.0 - current_conduction).max(0.1);
                let heat_generated = current_charge * current_charge * resistance * 0.0125;
                self.temperatures[idx] += heat_generated;
            }

            if !is_super_conductor {
                let decay_factor = if is_battery {
                    0.9999999
                } else if current_conduction > 0.5 {
                    0.95
                } else {
                    0.4
                };
                new_charge[idx] *= decay_factor;
            }
        }

        
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let current_charge = old_charge[idx];
                if current_charge <= 0.1 { continue; }

                let current_elem = self.elements[idx];
                let current_def = self.definitions.get(&current_elem).unwrap();
                let current_conduction = current_def.electrical_conductivity;
                let is_conductor = current_conduction > 0.5;

                let mut neighbors = [(x as i32 - 1, y as i32), (x as i32 + 1, y as i32), (x as i32, y as i32 - 1), (x as i32, y as i32 + 1)];
                neighbors.shuffle(&mut rng);

                let max_branches = if is_conductor { 4 } else { 1 };
                let mut branches_created = 0;

                for (nx, ny) in neighbors {
                    if !self.is_in_bounds(nx, ny) { continue; }
                    if branches_created >= max_branches { break; }

                    let n_idx = self.get_index(nx as usize, ny as usize);
                    let target_elem = self.elements[n_idx];
                    let target_def = self.definitions.get(&target_elem).unwrap();
                    let target_cond = target_def.electrical_conductivity;

                    let jump_chance = if target_cond > 0.1 {
                        1.0
                    } else if current_charge > 0.8 {
                        0.15
                    } else {
                        0.0
                    };

                    if rand::random::<f32>() < jump_chance {
                        let loss = if target_cond > 0.1 { 0.9 } else { 0.7 };
                        let transfer = current_charge * loss;

                        if transfer > new_charge[n_idx] {
                            new_charge[n_idx] = transfer;
                            branches_created += 1;
                        }
                    }
                }
            }
        }

        self.electrical_charge = new_charge;
    }

    pub fn equalize_superconductors(&mut self) {
        
    }

    fn update_force(&mut self) {
        
        
        
        for y in (0..self.height).rev() { 
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let (fx, fy) = self.force[idx];
                let fx_abs = fx.abs();
                let fy_abs = fy.abs();
                
                if fx_abs < 0.1 && fy_abs < 0.1 { 
                    self.force[idx] = (0.0, 0.0); 
                    continue; 
                }

                if fx_abs > 3.0 || fy_abs > 3.0 {
                    self.temperatures[idx] += (fx_abs + fy_abs) * 0.5;
                }

                let target_x = (x as f32 + fx).round() as i32;
                let target_y = (y as f32 + fy).round() as i32;

                if self.is_in_bounds(target_x, target_y) {
                    let target_idx = self.get_index(target_x as usize, target_y as usize);
                    let current_elem = self.elements[idx];
                    let target_elem = self.elements[target_idx];

                    if self.can_displace(&current_elem, &target_elem) {
                        
                        self.swap(x, y, target_x as usize, target_y as usize);
                        
                        
                        
                        self.force[target_idx] = (fx * 0.9, fy * 0.9); 
                        self.force[idx] = (0.0, 0.0); 
                    } else {
                        
                        
                        self.force[idx] = (fx * 0.3, fy * 0.3);
                    }
                } else {
                    
                    self.force[idx] = (0.0, 0.0);
                }
            }
        }
    }

    

    pub fn update_special_electronics(&mut self) {

    }

}


impl World {
    fn move_sand(&mut self, x: usize, y: usize, current: Element, order: [isize; 2]) -> bool {
        if self.try_swap(x, y, x, y + 1, current) {
            return true;
        }

        for dx in order {
            if let Some(tx) = x.checked_add_signed(dx) {
                if self.try_swap(x, y, tx, y + 1, current) {
                    return true;
                }
            }
        }

        false
    }

    fn move_water(&mut self, x: usize, y: usize, current: Element, order: [isize; 2]) -> bool {
        
        if self.try_swap(x, y, x, y + 1, current) {
            return true;
        }

        for &dx in &order {
            if let Some(tx) = x.checked_add_signed(dx) {
                if self.try_swap(x, y, tx, y + 1, current) {
                    return true;
                }
            }
        }

        
        
        let dispersion = 5;
        for &dx in &order {
            for d in 1..=dispersion {
                if let Some(tx) = x.checked_add_signed(dx * d) {
                    
                    if !self.is_in_bounds(tx as i32, y as i32)
                        || !self.can_displace(&current, &self.get(tx, y))
                    {
                        break;
                    }

                    
                    if self.is_in_bounds(tx as i32, (y + 1) as i32)
                        && self.can_displace(&current, &self.get(tx, y + 1))
                    {
                        return self.try_swap(x, y, tx, y + 1, current);
                    }

                    
                    if d == dispersion {
                        return self.try_swap(x, y, tx, y, current);
                    }
                }
            }
        }
        false
    }

    fn move_gas(&mut self, x: usize, y: usize, current: Element) -> bool {
        let mut rng = rand::thread_rng();

        
        
        let drift = if rand::random::<f32>() < 0.3 {
            if rand::random() { -1isize } else { 1isize }
        } else {
            0isize
        };

        if let Some(tx) = x.checked_add_signed(drift) {
            if y > 0 && self.try_swap(x, y, tx, y - 1, current) {
                return true;
            }
        }

        
        
        let waft_dist = 2;
        let mut horizontal_dirs = [-1isize, 1isize];
        use rand::seq::SliceRandom;
        horizontal_dirs.shuffle(&mut rng);

        for dx in horizontal_dirs {
            for d in 1..=waft_dist {
                if let Some(tx) = x.checked_add_signed(dx * d) {
                    if !self.try_swap(x, y, tx, y, current) {
                        break;
                    }
                    return true;
                }
            }
        }

        false
    }
}

impl World {
    
    #[inline]
    fn get_neighbors(&self, x: usize, y: usize) -> [Element; 4] {
        [
            if x > 0 { self.get(x - 1, y) } else { 1 },
            if x < self.width - 1 { self.get(x + 1, y) } else { 1 },
            if y > 0 { self.get(x, y - 1) } else { 1 },
            if y < self.height - 1 { self.get(x, y + 1) } else { 1 },
        ]
    }

    
    fn find_nearest_air(&self, x: usize, y: usize, max_radius: i32) -> Option<(usize, usize)> {
        if !self.is_in_bounds(x as i32, y as i32) {
            return None;
        }

        if self.get(x, y) == 0 {
            return Some((x, y));
        }

        use std::collections::VecDeque;

        let mut visited = vec![false; self.width * self.height];
        let mut q: VecDeque<(i32, i32, i32)> = VecDeque::new();
        q.push_back((x as i32, y as i32, 0));
        visited[self.get_index(x, y)] = true;

        while let Some((cx, cy, dist)) = q.pop_front() {
            if dist >= max_radius {
                continue;
            }

            
            for (dx, dy) in &[(0, -1), (0, 1), (-1, 0), (1, 0)] {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }
                let n_idx = self.get_index(nx as usize, ny as usize);
                if visited[n_idx] {
                    continue;
                }
                visited[n_idx] = true;

                if self.get(nx as usize, ny as usize) == 0 {
                    return Some((nx as usize, ny as usize));
                }

                q.push_back((nx, ny, dist + 1));
            }
        }

        None
    }

    fn check_condition(&self, x: usize, y: usize, cond: &Condition) -> bool {
        let idx = self.get_index(x, y);
        match cond {
            
            Condition::LifetimeGreater(t) => self.lifetimes[idx] > *t,
            Condition::TemperatureAbove(t) => self.temperatures[idx] > *t,
            Condition::TemperatureBelow(t) => self.temperatures[idx] < *t,
            Condition::RandomChance(p) => rand::random::<f32>() < *p,
            Condition::HasChargeAbove(charge) => self.electrical_charge[idx] > *charge,
            Condition::HasChargeBelow(charge) => self.electrical_charge[idx] < *charge,
            
            
            Condition::NearElement(target_elem) => {
                let neighbors = self.get_neighbors(x, y);
                neighbors.iter().any(|e| e == target_elem)
            }
            Condition::NotNearElement(target_elem) => {
                let neighbors = self.get_neighbors(x, y);
                !neighbors.iter().any(|e| e == target_elem)
            }
            Condition::NearElementType(target_elem) => {
                let neighbors = self.get_neighbors(x, y);
                neighbors.iter().any(|e| self.definitions.get(e).unwrap().movement == *target_elem)
            }
            Condition::NearTemperatureAbove(t) => {
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let n_idx = self.get_index(nx as usize, ny as usize);
                        if self.temperatures[n_idx] > *t {
                            return true;
                        }
                    }
                }
                false
            }
            Condition::NearTemperatureBelow(t) => {
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let n_idx = self.get_index(nx as usize, ny as usize);
                        if self.temperatures[n_idx] < *t {
                            return true;
                        }
                    }
                }
                false
            }
            Condition::IsInsideOf(target_elem) => {
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        if self.get(nx as usize, ny as usize) != *target_elem {
                            return false; 
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            Condition::IsNotInsideOf(target_elem) => {
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        if self.get(nx as usize, ny as usize) != *target_elem {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
                false
            }
            
            
            Condition::IsElementInRadius(target_elem, radius) => {
                
                let neighbors = self.get_neighbors(x, y);
                if neighbors.iter().any(|e| e == target_elem) {
                    return true;
                }
                
                
                let frames_since_check = self.frame_count - self.last_expensive_check[idx];
                if frames_since_check < 3 {
                    
                    
                    return false;
                }
                
                
                let r = *radius as i32;
                let mut found = false;
                for ring in 1..=r {
                    for dy in -ring..=ring {
                        for dx in -ring..=ring {
                            if dx.abs() != ring && dy.abs() != ring {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                                if self.get(nx as usize, ny as usize) == *target_elem {
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found { break; }
                    }
                    if found { break; }
                }
                found
            }
        }
    }

    pub fn process_reactions(&mut self, x: usize, y: usize) {
        let idx = self.get_index(x, y);
        let current_elem = self.elements[idx];
        let current_def = self.definitions.get(&current_elem).unwrap();
        let local_time = 1.0 + self.time_effects[idx]; 

        
        if local_time <= 0.0 {
            return;
        }

        /*
        let resistance = current_def

        
        if resistance < 1.0 {
            
            let neighbors = [
                (x as i32, y as i32 - 1),
                (x as i32, y as i32 + 1),
                (x as i32 - 1, y as i32),
                (x as i32 + 1, y as i32),
            ];

            for (nx, ny) in neighbors {
                if self.is_in_bounds(nx, ny) {
                    let neighbor_elem = self.get(nx as usize, ny as usize);

                    if neighbor_elem.is_corrosive() {
                        let corrosion_chance = (0.2 * local_time) * (1.0 - resistance);

                        if rand::random::<f32>() < corrosion_chance {
                            self.set(x, y, Element::Smoke);
                            
                            if rand::random::<f32>() < 0.5 {
                                self.set(nx as usize, ny as usize, 0);
                            }
                            return; 
                        }
                    }
                }
            }
        }
         */

        
        if current_elem == 39 {
            let blast_radius = 5;
            let force_magnitude = 4.0;
            
            for dy in -(blast_radius as i32)..=blast_radius as i32 {
                for dx in -(blast_radius as i32)..=blast_radius as i32 {
                    if dx == 0 && dy == 0 { continue; }
                    
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if self.is_in_bounds(nx, ny) {
                        let n_idx = self.get_index(nx as usize, ny as usize);
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        
                        if dist <= blast_radius as f32 {
                            let force_falloff = 1.0 - (dist / blast_radius as f32);
                            let normalized_dx = dx as f32 / dist;
                            let normalized_dy = dy as f32 / dist;
                            
                            self.force[n_idx].0 += normalized_dx * force_magnitude * force_falloff;
                            self.force[n_idx].1 += normalized_dy * force_magnitude * force_falloff;
                        }
                    }
                }
            }
        }
        
        if current_elem == 0 {
            let has_fire = self.check_condition(x, y, &Condition::NearElement(5));
            let neighbors = [(x as i32, y as i32 - 1), (x as i32, y as i32 + 1), (x as i32 - 1, y as i32), (x as i32 + 1, y as i32)];
            
            if has_fire {
                for (nx, ny) in neighbors {
                    if self.is_in_bounds(nx, ny) {
                        let neighbor_elem = self.get(nx as usize, ny as usize);
                        let neighbor_def = self.definitions.get(&neighbor_elem).unwrap();
                        let flammibility = neighbor_def.flammability;
                        if flammibility > 0.0 {
                            if rand::random::<f32>() < (flammibility * local_time) {
                                self.set(x, y, 5);
                                return;
                            }
                        }
                    }
                }
            }
        }

        
        let loops = local_time.ceil() as i32;
        for _ in 0..loops {
            let elem_idx = current_elem as usize;
            
            
            let output_items: Option<Vec<_>> = self.reaction_lookup[elem_idx]
                .iter()
                .find(|r| r.conditions.iter().all(|c| self.check_condition(x, y, c)))
                .map(|r| r.output.clone());

            if let Some(output_items) = output_items {
                
                for (output_elem, count) in &output_items {
                    let mut placed = 0;

                    for _ in 0..*count {
                        
                        if self.get(x, y) == current_elem {
                            self.set(x, y, *output_elem);
                            placed += 1;
                            continue;
                        }

                        
                        let neighbors = [
                            (x as i32, y as i32 - 1), (x as i32, y as i32 + 1),
                            (x as i32 - 1, y as i32), (x as i32 + 1, y as i32),
                        ];

                        let mut placed_here = false;
                        for (nx, ny) in neighbors {
                            if placed_here { break; }
                            if !self.is_in_bounds(nx, ny) { continue; }
                            if self.get(nx as usize, ny as usize) == 0 {
                                self.set(nx as usize, ny as usize, *output_elem);
                                placed += 1;
                                placed_here = true;
                            }
                        }
                        if placed_here { continue; }

                        
                        if let Some((tx, ty)) = self.find_nearest_air(x, y, 6) {
                            
                            self.set(tx, ty, *output_elem);
                            placed += 1;
                            continue;
                        }

                        
                    }
                }
                
                break;
            }
        }
    }
}

pub struct PixelInfo {
    pub element: Element,
    pub name: String,
    pub temp: f32,
    pub age: u32,
    pub time_mod: f32,
    pub reactions: Vec<Reaction>,
    pub flammability: f32,
    pub density: f32,
    pub charge: f32,
    pub thermal_conductivity: f32,
    pub electrical_conductivity: f32,
}

impl World {
    pub fn get_pixel_info(&self, x: usize, y: usize) -> Option<PixelInfo> {
        if !self.is_in_bounds(x as i32, y as i32) {
            return None;
        }

        let idx = self.get_index(x, y);
        let elem = self.elements[idx];
        let elem_def = self.definitions.get(&elem).unwrap();
        Some(PixelInfo {
            element: elem,
            name: elem_def.name.clone(),
            temp: self.temperatures[idx],
            age: self.lifetimes[idx],
            time_mod: 1.0 + self.time_effects[idx],
            reactions: self.reaction_lookup[elem as usize].clone(),
            flammability: elem_def.flammability,
            density: elem_def.density,
            charge: self.electrical_charge[idx],
            thermal_conductivity: elem_def.thermal_conductivity,
            electrical_conductivity: elem_def.electrical_conductivity,
        })
    }
}

impl World {
    fn is_touching_solid(&self, x: usize, y: usize) -> bool {
        let neighbors = [
            (x as i32 - 1, y as i32), (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1), (x as i32, y as i32 + 1),
        ];

        for (nx, ny) in neighbors {
            if self.is_in_bounds(nx, ny) {
                let e = self.get(nx as usize, ny as usize);
                let e_def = self.definitions.get(&e).unwrap();
                if e_def.movement != Movement::Gas && e_def.movement != Movement::Liquid {
                    return true;
                }
            }
        }
        false
    }

    fn can_displace(&self, a: &Element, b: &Element) -> bool {
        let a_def = self.definitions.get(a).unwrap();
        let b_def = self.definitions.get(b).unwrap();

        if *a == 0 {
            return false;
        }

        let source_move = a_def.movement;
        
        let target_move = b_def.movement;
        if source_move == Movement::Static || target_move == Movement::Static {
            return false;
        }

        let source_density = a_def.density;
        let target_density = b_def.density;
        match source_move {
            
            
            Movement::Gas => {
                let is_target_solid = target_move == Movement::Static || target_move == Movement::Powder;
                !is_target_solid && source_density < target_density
            },
            
            _ => source_density > target_density,
        }
    }
}

impl World {
    pub fn save_discoveries(&self, _filename: &str) -> std::io::Result<()> {
        let mut data = String::new();
        for elem in &self.discovered_elements {
            data.push_str(&elem.to_string());
            data.push('\n');
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut file = std::fs::File::create(_filename)?;
            
            file.write_all(data.as_bytes())?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(local_storage)) = window.local_storage() {
                    let _ = local_storage.set_item("dust_sim_discoveries", &data);
                }
            }
        }
        Ok(())
    }

    pub fn load_discoveries(&mut self, _filename: &str) {
        let mut raw_data = String::new();

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(content) = std::fs::read_to_string(_filename) {
                raw_data = content;
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(local_storage)) = window.local_storage() {
                    if let Ok(Some(content)) = local_storage.get_item("dust_sim_discoveries") {
                        raw_data = content;
                    }
                }
            }
        }

        
        for line in raw_data.lines() {
            if let Ok(elem) = line.parse::<Element>() {
                self.discovered_elements.insert(elem);
            }
        }
    }
}