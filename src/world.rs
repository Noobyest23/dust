pub mod element;
use element::{Condition, Element, Movement, Reaction};
use strum::IntoEnumIterator;
use std::cmp::min;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

#[cfg(target_arch = "wasm32")]
use web_sys::{Storage, Window};

use crate::world::element::can_displace;

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
    
    pub discovered_elements: HashSet<Element>,
    pub new_discovery: Option<Element>,
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
            return Element::Stone;
        }
        self.elements[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, element: Element) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.elements[idx] = element;
            self.last_update[idx] = self.frame_count;

            self.shades[idx] = if element == Element::Air {
                0
            } else {
                rand::random::<u8>()
            };
            self.velocities[idx] = if rand::random() { 1 } else { -1 };
            self.lifetimes[idx] = 0;
            self.temperatures[idx] = element.base_temp();

            if !self.discovered_elements.contains(&element) && !element.is_super_hidden() {
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

            
            
            self.last_update[idx1] = self.frame_count;
            self.last_update[idx2] = self.frame_count;
        }
    }

    fn try_swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, current: Element) -> bool {
        if x2 >= self.width || y2 >= self.height {
            return false;
        }

        let target = self.get(x2, y2);
        if can_displace(current, target) {
            self.swap(x1, y1, x2, y2);
            return true;
        }

        false
    }
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let mut reaction_lookup = Vec::new();
        let mut discovered_elements = HashSet::new();

        for elem in Element::iter() {
            reaction_lookup.push(elem.get_reactions());
            
            
            if !elem.is_hidden() && !elem.is_super_hidden() {
                discovered_elements.insert(elem);
            }
        }
        Self {
            width,
            height,
            elements: vec![Element::Air; width * height],
            shades: vec![0; width * height],
            velocities: vec![0; width * height],
            temperatures: vec![20.0 as f32; width * height],
            last_update: vec![0; width * height],
            frame_count: 0,
            lifetimes: vec![0; width * height],
            ambient_temp: 20.0,
            reaction_lookup,
            time_effects: vec![0.0; width * height],
            electrical_charge: vec![0.0; width * height],
            force: vec![(0.0, 0.0); width * height],
            
            discovered_elements,
            new_discovery: None,
        }
    }
}


impl World {
    pub fn update(&mut self) {
        self.frame_count += 1;
        self.update_heat();
        self.update_physics();
        self.update_time_field();
        self.update_electrical_field();
        

        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                if self.elements[idx] == Element::Air && self.temperatures[idx] == self.ambient_temp
                {
                    continue; 
                }
                self.process_reactions(x, y);
            }
        }

        
        
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

                
                if current == Element::Air {
                    continue;
                }

                self.process_reactions(x, y);

                if current.movement_type() == Movement::Gas {
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
                let move_type = current.movement_type();

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
        let mut growth_actions = Vec::new(); 

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let current_elem = self.elements[idx];
                let local_time = 1.0 + self.time_effects[idx];

                if current_elem == Element::Electricity {
                    self.electrical_charge[idx] = 100.0;
                }

                if current_elem != Element::Air {
                    self.lifetimes[idx] += local_time.max(0.0) as u32;
                    
                    

                    
                    if let Some((min_temp, max_temp)) = current_elem.growth_range() {
                        let temp = self.temperatures[idx];
                        
                        
                        
                        if temp >= min_temp && temp <= max_temp && rand::random::<f32>() < (0.0005 * local_time) {
                            
                            
                            let neighbors = [
                                (x as i32 - 1, y as i32), (x as i32 + 1, y as i32),
                                (x as i32, y as i32 - 1), (x as i32, y as i32 + 1),
                            ];

                            for (nx, ny) in neighbors {
                                if self.is_in_bounds(nx, ny) {
                                    let n_idx = self.get_index(nx as usize, ny as usize);
                                    
                                    
                                    if self.elements[n_idx] == Element::Air {
                                        
                                        
                                        if self.is_touching_solid(nx as usize, ny as usize) {
                                            growth_actions.push((nx as usize, ny as usize, current_elem));
                                            break; 
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        
        for (gx, gy, elem) in growth_actions {
            self.set(gx, gy, elem);
        }
    }

    pub fn update_heat(&mut self) {
        let ambient = self.ambient_temp;
        
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let current_elem = self.elements[idx];
                let current_temp = self.temperatures[idx];

                
                if let Some(temp) = current_elem.source_temp() {
                    self.temperatures[idx] = temp;
                    self.radiate_heat(x, y, temp);
                    continue;
                }

                
                
                self.conduct_heat(x, y, current_elem, current_temp);

                
                
                let local_time = 1.0 + self.time_effects[idx];
                let base_rate = if current_elem == Element::Air { 0.1 } else { 0.01 };

                
                let cooling_rate = (base_rate * local_time).clamp(0.0, 0.5);

                let temp_diff = ambient - current_temp;
                if !temp_diff.is_nan() {
                    self.temperatures[idx] += temp_diff * cooling_rate;
                }

                
                if y > 0 && current_temp > ambient + 5.0 {
                    let up_idx = self.get_index(x, y - 1);
                    let temp_up = self.temperatures[up_idx];
                    let diff = current_temp - temp_up;
                    
                    
                    let mut transfer = diff * 0.1 * local_time;
                    let max_transfer = diff * 0.5; 
                    
                    if transfer > max_transfer {
                        transfer = max_transfer;
                    }

                    if transfer > 0.0 && !transfer.is_nan() {
                        self.temperatures[up_idx] += transfer;
                        self.temperatures[idx] -= transfer;
                    }
                }

            }
        }
    }

    
    fn radiate_heat(&mut self, x: usize, y: usize, temp: f32) {
        let neighbors = [
            (x as i32 - 1, y as i32),
            (x as i32 + 1, y as i32),
            (x as i32, y as i32 - 1),
            (x as i32, y as i32 + 1),
        ];
        for (nx, ny) in neighbors {
            if self.is_in_bounds(nx, ny) {
                let n_idx = self.get_index(nx as usize, ny as usize);
                
                let diff = temp - self.temperatures[n_idx];
                self.temperatures[n_idx] += diff * 0.1;
            }
        }
    }

    fn conduct_heat(&mut self, x: usize, y: usize, current_elem: Element, current_temp: f32) {
        let idx_a = self.get_index(x, y);
        let conductivity_a = current_elem.conductivity();
        
        
        let dt = 1.0 + self.time_effects[idx_a];

        for (nx, ny) in [(x + 1, y), (x, y + 1)] {
            if nx < self.width && ny < self.height {
                let idx_b = self.get_index(nx, ny);
                let elem_b = self.elements[idx_b];
                let temp_b = self.temperatures[idx_b];

                let diff = current_temp - temp_b;
                
                if diff.abs() > 0.01 {
                    let combined_conductivity = (conductivity_a + elem_b.conductivity()) * 0.5;

                    
                    
                    
                    let mut transfer_rate = combined_conductivity * dt;
                    
                    
                    if transfer_rate > 0.5 {
                        transfer_rate = 0.5;
                    }

                    let transfer_amount = diff * transfer_rate;

                    
                    self.temperatures[idx_a] -= transfer_amount;
                    self.temperatures[idx_b] += transfer_amount;
                }
            }
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
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);

                
                self.time_effects[idx] *= 0.995;

                
                if x > 0 && x < self.width - 1 {
                    let left = self.get_index(x - 1, y);
                    let right = self.get_index(x + 1, y);
                    let avg = (self.time_effects[left] + self.time_effects[right]) * 0.5;
                    self.time_effects[idx] = self.time_effects[idx] * 0.8 + avg * 0.2;
                }
            }
        }
    }

    pub fn update_electrical_field(&mut self) {
        let mut new_charge = self.electrical_charge.clone();
        let mut visited = vec![false; (self.width * self.height) as usize];

        
        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                if self.elements[idx].conductivity() > 1.0 && !visited[idx] {
                    let mut cluster = Vec::new();
                    let mut stack = vec![(x, y)];
                    visited[idx] = true;
                    let mut total_charge = 0.0;

                    while let Some((cx, cy)) = stack.pop() {
                        let c_idx = self.get_index(cx, cy);
                        cluster.push(c_idx);
                        total_charge += self.electrical_charge[c_idx];

                        for (nx, ny) in [(cx as i32 - 1, cy as i32), (cx as i32 + 1, cy as i32), (cx as i32, cy as i32 - 1), (cx as i32, cy as i32 + 1)] {
                            if self.is_in_bounds(nx, ny) {
                                let n_idx = self.get_index(nx as usize, ny as usize);
                                if !visited[n_idx] && self.elements[n_idx].conductivity() > 1.0 {
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
                            
                            self.electrical_charge[pixel_idx] = average_charge;
                        }
                    }
                }
            }
        }

        
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let current_charge = self.electrical_charge[idx];
                if current_charge <= 0.1 { continue; }
                

                let current_elem = self.elements[idx];
                let current_conduction = current_elem.conductivity();

                self.temperatures[idx] += current_charge * if (current_conduction < 1.0) {current_conduction} else {0.9999};

                let is_conductor = current_conduction > 0.5;
                let is_super_conductor = current_conduction > 1.0;
                let is_battery = current_elem == Element::Battery;

                if !is_super_conductor {
                    let decay_factor = if is_battery {
                        0.9999999 
                    } else if current_elem.conductivity() > 0.5 {
                        0.95   
                    } else {
                        0.4    
                    };
                    new_charge[idx] *= decay_factor;
                }

                
                let mut neighbors = [(x as i32 - 1, y as i32), (x as i32 + 1, y as i32), (x as i32, y as i32 - 1), (x as i32, y as i32 + 1)];
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                neighbors.shuffle(&mut rng);

                let max_branches = if is_conductor { 4 } else { 1 }; 
                let mut branches_created = 0;

                for (nx, ny) in neighbors {
                    if !self.is_in_bounds(nx, ny) { continue; }
                    if branches_created >= max_branches { break; }

                    let n_idx = self.get_index(nx as usize, ny as usize);
                    let target_elem = self.elements[n_idx];
                    let target_cond = target_elem.conductivity();

                    
                    
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
        let mut visited = vec![false; (self.width * self.height) as usize];
        let mut new_charge = self.electrical_charge.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);

                
                
            }
        }
        self.electrical_charge = new_charge;
    }

    fn update_force(&mut self) {
        
        
        
        for y in (0..self.height).rev() { 
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                let (fx, fy) = self.force[idx];

                
                if fx.abs() < 0.1 && fy.abs() < 0.1 { 
                    self.force[idx] = (0.0, 0.0); 
                    continue; 
                }

                
                let target_x = (x as f32 + fx).round() as i32;
                let target_y = (y as f32 + fy).round() as i32;

                if self.is_in_bounds(target_x, target_y) {
                    let target_idx = self.get_index(target_x as usize, target_y as usize);
                    let current_elem = self.elements[idx];
                    let target_elem = self.elements[target_idx];

                    if can_displace(current_elem, target_elem) {
                        
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
                        || !can_displace(current, self.get(tx, y))
                    {
                        break;
                    }

                    
                    if self.is_in_bounds(tx as i32, (y + 1) as i32)
                        && can_displace(current, self.get(tx, y + 1))
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
    fn check_condition(&self, x: usize, y: usize, cond: &Condition) -> bool {
        let idx = self.get_index(x, y);
        match cond {
            Condition::LifetimeGreater(t) => self.lifetimes[idx] > *t,
            Condition::TemperatureAbove(t) => self.temperatures[idx] > *t,
            Condition::TemperatureBelow(t) => self.temperatures[idx] < *t,
            Condition::RandomChance(p) => rand::random::<f32>() < *p,
            Condition::NearElement(target_elem) => {
                
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        if self.get(nx as usize, ny as usize) == *target_elem {
                            return true;
                        }
                    }
                }
                false
            }
            Condition::NotNearElement(target_elem) => {
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        if self.get(nx as usize, ny as usize) == *target_elem {
                            return false;
                        }
                    }
                }
                true
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
                
                let mut surrounded = true;
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        if self.get(nx as usize, ny as usize) != *target_elem {
                            surrounded = false;
                            break;
                        }
                    } else {
                        surrounded = false; 
                        break;
                    }
                }
                !surrounded
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
            Condition::IsElementInRadius(target_elem, radius) => {
                let r = *radius as i32;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx == 0 && dy == 0 {
                            continue;
                        } 
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            if self.get(nx as usize, ny as usize) == *target_elem {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            Condition::HasChargeAbove(charge) => {
                self.electrical_charge[idx] > *charge
            }
            Condition::HasChargeBelow(charge) => {
                self.electrical_charge[idx] < *charge
            }
            Condition::NearElementType(target_elem) => {
                
                for (nx, ny) in [
                    (x as i32, y as i32 - 1),
                    (x as i32, y as i32 + 1),
                    (x as i32 - 1, y as i32),
                    (x as i32 + 1, y as i32),
                ] {
                    if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                        let elem = self.get(nx as usize, ny as usize);
                        if elem == Element::Air {
                            continue;
                        }
                        if elem.movement_type() == *target_elem {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    pub fn process_reactions(&mut self, x: usize, y: usize) {
        let idx = self.get_index(x, y);
        let current_elem = self.elements[idx];

        let local_time = 1.0 + self.time_effects[idx]; 

        
        if local_time <= 0.0 {
            return;
        }

        
        let resistance = current_elem.corrosive_resistance();

        
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
                                self.set(nx as usize, ny as usize, Element::Air);
                            }
                            return; 
                        }
                    }
                }
            }
        }
        

        
        if current_elem == Element::Air {
            let has_fire = self.check_condition(x, y, &Condition::NearElement(Element::Fire));
            let neighbors = [(x as i32, y as i32 - 1), (x as i32, y as i32 + 1), (x as i32 - 1, y as i32), (x as i32 + 1, y as i32)];
            
            if has_fire {
                for (nx, ny) in neighbors {
                    if self.is_in_bounds(nx, ny) {
                        let neighbor_elem = self.get(nx as usize, ny as usize);
                        
                        let flammibility = neighbor_elem.flammability();
                        if flammibility > 0.0 {
                            if rand::random::<f32>() < (flammibility * local_time) {
                                self.set(x, y, Element::Fire);
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
            let reactions = &self.reaction_lookup[elem_idx];

            
            if reactions.is_empty() {
                return;
            }

            
            if let Some(reaction) = reactions
                .iter()
                .find(|r| r.conditions.iter().all(|c| self.check_condition(x, y, c)))
            {
                self.set(x, y, reaction.output);
            }
        }
    }
}

pub struct PixelInfo {
    pub element: Element,
    pub temp: f32,
    pub age: u32,
    pub time_mod: f32,
    pub reactions: Vec<Reaction>,
    pub flammability: f32,
    pub corrosion_resistance: f32,
    pub density: i32,
    pub charge: f32,
    pub conductivity: f32,
}

impl World {
    pub fn get_pixel_info(&self, x: usize, y: usize) -> Option<PixelInfo> {
        if !self.is_in_bounds(x as i32, y as i32) {
            return None;
        }

        let idx = self.get_index(x, y);
        Some(PixelInfo {
            element: self.elements[idx],
            temp: self.temperatures[idx],
            age: self.lifetimes[idx],
            time_mod: 1.0 + self.time_effects[idx],
            reactions: self.reaction_lookup[self.elements[idx] as usize].clone(),
            flammability: self.elements[idx].flammability(),
            corrosion_resistance: self.elements[idx].corrosive_resistance(),
            density: self.elements[idx].density(),
            charge: self.electrical_charge[idx],
            conductivity: self.elements[idx].conductivity(),
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
                
                if e.movement_type() != Movement::Gas && e.movement_type() != Movement::Liquid {
                    return true;
                }
            }
        }
        false
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