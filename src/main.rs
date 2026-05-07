mod world;
use std::cmp::min;

use strum::{EnumCount, IntoEnumIterator};
use world::{World};
use world::element::Element; 

use eframe::egui;
use eframe::egui::{Color32, ColorImage, TextureOptions};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;


#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    web_time::Instant,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Power {
    Heat,
    Cold,
    Charge,
    Catalyst,
    Stasis,
    Cleaner,
    Push
}

impl Power {
    fn iter() -> impl Iterator<Item = (Power, &'static str, Color32)> {
        [
            (Power::Heat, "Heat", Color32::from_rgb(255, 100, 0)),
            (Power::Cold, "Cold", Color32::from_rgb(0, 150, 255)),
            (Power::Charge, "Charge", Color32::from_rgb(255, 255, 0)),
            (Power::Catalyst, "Catalyst", Color32::from_rgb(150, 0, 255)),
            (Power::Stasis, "Stasis", Color32::from_rgb(100, 255, 100)),
            (Power::Cleaner, "Cleaner", Color32::from_rgb(200, 200, 200)),
            (Power::Push, "Push", Color32::from_rgb(255, 150, 150)),
        ]
        .into_iter()
    }
}


struct DustApp {
    world: World,
    width: usize,
    height: usize,
    selected_element: Element,
    selected_power: Power,
    brush_size: i32,
    paused: bool,
    scale: f32,
    pub scroll_accumulator: f32,
    last_frame_time: Instant,
    fps: f64,
    sim_rect: egui::Rect,
    texture: Option<egui::TextureHandle>,
    game_speed: i8,
	discovery_timer: f32,          
    last_discovered: Option<Element>,
}

impl DustApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = 375usize;
        let height = 200usize;

		let mut world = World::new(width, height);
		world.load_discoveries("discovered_elements.txt");
        Self {
            world,
            width,
            height,
            selected_element: Element::Sand,
            selected_power: Power::Heat,
            brush_size: 2,
            paused: false,
            scale: 2.0,
            scroll_accumulator: 0.0,
            last_frame_time: Instant::now(),
            fps: 0.0,
            sim_rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(width as f32, height as f32)),
            texture: None,
            game_speed: 2,
			discovery_timer: 0.0,
			last_discovered: None,
        }
    }

    fn world_to_color_image(&self) -> ColorImage {
        let mut pixels = vec![Color32::BLACK; self.width * self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let elem = self.world.get(x, y);
                let shade = self.world.get_shade(x, y);
                let base: [u8; 4] = elem.color();

                let color = if elem == Element::Air {
                    
                    base
                } else {
                    noise_color(base, shade, elem as u32)
                };

                let mut final_color =
                    Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);

				

                
                
                

                
                let temp = self.world.temperatures[idx] - 20.0;
                if temp > 1.0 {
                    
                    let heat_glow = (temp as u8).min(150);
                    final_color = Color32::from_rgb(
                        final_color.r().saturating_add(heat_glow),
                        final_color.g(),
                        final_color.b(),
                    );
                } else if temp < -1.0 {
                    
                    let cold_glow = ((-temp) as u8).min(150);
                    final_color = Color32::from_rgb(
                        final_color.r(),
                        final_color.g(),
                        final_color.b().saturating_add(cold_glow),
                    );
                }

               let time_val = self.world.time_effects[idx];
                
                let slow_time = self.world.frame_count as f32 * 0.03;

                if time_val > 0.1 {
                    
                    let pulse = slow_time.sin().abs(); 
                    let intensity = (time_val.min(10.0) * 15.0 * pulse) as u8; 
                    
                    final_color = Color32::from_rgb(
                        final_color.r().saturating_add(intensity),
                        final_color.g(),
                        final_color.b().saturating_add(intensity),
                    );
                } else if time_val < -0.1 {
                    
                    
                    
                    let pulse = slow_time.cos().abs(); 
                    let intensity = (time_val.abs().min(10.0) * 15.0 * pulse) as u8;

                    final_color = Color32::from_rgb(
                        final_color.r(),
                        final_color.g().saturating_add(intensity),
                        final_color.b().saturating_add(intensity),
                    );
                }

                let charge = self.world.electrical_charge[idx];
                if charge > 0.1 {
                    let intensity = min(100, (charge * 10.0) as u8); 
                    
                    final_color = Color32::from_rgb(
                        final_color.r().saturating_add(intensity),
                        final_color.g().saturating_add(intensity),
                        final_color.b(),
                    );
                }

				let force = self.world.force[idx];
				let force_magnitude = (force.0.powi(2) + force.1.powi(2)).sqrt();
				if force_magnitude > 0.1 {
					let intensity = (force_magnitude.min(10.0) * 20.0) as u8; 

					final_color = Color32::from_rgb(
						final_color.r().saturating_add(intensity),
						final_color.g().saturating_add(intensity),
						final_color.b().saturating_add(intensity),
					);
				}

                pixels[idx] = final_color;
            }
        }
        
        ColorImage {
            size: [self.width, self.height],
            pixels,
            source_size: egui::vec2(self.width as f32, self.height as f32),
        }
    }

fn _ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        
        let available_size = ui.available_size();

        
        let world_aspect = self.width as f32 / self.height as f32;
        let window_aspect = available_size.x / available_size.y;

        let display_size = if window_aspect > world_aspect {
            
            egui::vec2(available_size.y * world_aspect, available_size.y)
        } else {
            
            egui::vec2(available_size.x, available_size.x / world_aspect)
        };

        
        let display_size = display_size * 0.98;

        
        self.scale = display_size.x / self.width as f32;

        
        ui.vertical_centered(|ui| {
            ui.label("Simulation (click/drag to paint)");

            let color_image = self.world_to_color_image();
            
            
            let texture_handle = self.texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "sim_tex",
                    color_image.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });
            
            
            texture_handle.set(color_image, egui::TextureOptions::NEAREST);

            
            let img_widget = egui::Image::new(&*texture_handle) 
                .fit_to_exact_size(display_size)
                .sense(egui::Sense::click_and_drag());

            let resp = ui.add(img_widget);
            self.sim_rect = resp.rect;

			if self.discovery_timer > 0.0 {
				if let Some(elem) = self.last_discovered {
					let alpha = (self.discovery_timer.min(1.0) * 255.0) as u8; 
					let painter = ui.painter_at(self.sim_rect);
					
					
					let center_top = self.sim_rect.center_top() + egui::vec2(0.0, 20.0);
					
					painter.text(
						center_top,
						egui::Align2::CENTER_TOP,
						format!("NEW DISCOVERY: {:?}", elem),
						egui::FontId::proportional(24.0),
						Color32::from_rgba_unmultiplied(255, 255, 0, alpha), 
					);
				}
			}

            
            if resp.dragged() || resp.clicked() {
    if let Some(pos) = resp.interact_pointer_pos() {
        let rect = resp.rect;
        let local = pos - rect.min;

        let px = (local.x / self.scale).floor() as i32;
        let py = (local.y / self.scale).floor() as i32;

        let is_primary = ui.input(|i| i.pointer.primary_down());
        let is_secondary = ui.input(|i| i.pointer.secondary_down());
        let shift_held = ui.input(|i| i.modifiers.shift);

        if is_primary && shift_held {
            
            for dy in -self.brush_size..self.brush_size {
                for dx in -self.brush_size..self.brush_size {
                    let x = px + dx;
                    let y = py + dy;

                    if x >= 0
                        && x < self.width as i32
                        && y >= 0
                        && y < self.height as i32
                    {
                        match self.selected_power {
                            Power::Heat => {
                                self.world.add_heat(x as usize, y as usize, 100.0 * (1.0 + (self.brush_size as f32 / 10.0)));
                            }
                            Power::Cold => {
                                self.world.add_cold(x as usize, y as usize, 100.0 * (1.0 + (self.brush_size as f32 / 10.0)));
                            }
                            Power::Charge => {
                                let idx = self.world.get_index(x as usize, y as usize);
                                self.world.electrical_charge[idx] += 1.0; 
                            }
                            Power::Catalyst => {
                                let idx = self.world.get_index(x as usize, y as usize);
                                self.world.time_effects[idx] += 5.0; 
                            }
                            Power::Stasis => {
                                let idx = self.world.get_index(x as usize, y as usize);
                                self.world.time_effects[idx] -= 2.0; 
                            }
                            Power::Cleaner => {
                                if (self.world.get(x as usize, y as usize) == self.selected_element) {
                                    self.world.set(x as usize, y as usize, Element::Air);
                                }
                            }
                            Power::Push => {
                                
                                let delta = ui.input(|i| i.pointer.delta());
                                let speed = delta.length();

                                
                                if speed > 0.1 {
                                    let idx = self.world.get_index(x as usize, y as usize);
                                    
                                    
                                    let force_multiplier = 0.25; 
                                    
                                    
                                    
                                    self.world.force[idx].0 += delta.x * force_multiplier;
                                    self.world.force[idx].1 += delta.y * force_multiplier;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            
            let paint_element = if is_secondary {
                Some(Element::Air)
            } else if is_primary {
                Some(self.selected_element)
            } else {
                None
            };

            if let Some(elem) = paint_element {
                for dy in -self.brush_size..self.brush_size {
                    for dx in -self.brush_size..self.brush_size {
                        let x = px + dx;
                        let y = py + dy;

                        if x >= 0
                            && x < self.width as i32
                            && y >= 0
                            && y < self.height as i32
                        {
                            let ux = x as usize;
                            let uy = y as usize;

                            if self.world.get(ux, uy) == Element::Air
                                || elem == Element::Air
                            {
                                self.world.set(ux, uy, elem);
                            }
                        }
                    }
                }
            }
        }
    }
}
        
            
        });
    }

    fn take_screenshot(&self) {
        let image = self.world_to_color_image();
        let size = image.size;
        
        
        let mut pixels = Vec::with_capacity(image.pixels.len() * 4);
        for color in image.pixels {
            pixels.push(color.r());
            pixels.push(color.g());
            pixels.push(color.b());
            pixels.push(color.a());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = format!("screenshot_{}.png", self.world.frame_count);
            
            
            if let Some(buffer) = image::RgbaImage::from_raw(size[0] as u32, size[1] as u32, pixels) {
                
                match buffer.save(&path) {
                    Ok(_) => println!("Screenshot saved to: {}", path),
                    Err(e) => eprintln!("Failed to save screenshot: {}", e),
                }
            } else {
                eprintln!("Failed to create image buffer: pixel data size mismatch.");
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    
                    let canvas = document.create_element("canvas").unwrap()
                        .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                    canvas.set_width(size[0] as u32);
                    canvas.set_height(size[1] as u32);
                    
                    let context = canvas.get_context("2d").unwrap().unwrap()
                        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

                    
                    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                        wasm_bindgen::Clamped(&pixels), size[0] as u32, size[1] as u32
                    ).unwrap();
                    
                    let _ = context.put_image_data(&image_data, 0.0, 0.0);
                    
                    
                    let data_url = canvas.to_data_url().unwrap();
                    let link = document.create_element("a").unwrap()
                        .dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                    link.set_href(&data_url);
                    link.set_download(&format!("dust_{}.png", self.world.frame_count));
                    link.click();
                }
            }
        }
    }
}

impl eframe::App for DustApp {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;
        
        
        if delta > 0.0 {
            self.fps = self.fps * 0.9 + (1.0 / delta) * 0.1;
        }

		
		let delta = now.duration_since(self.last_frame_time).as_secs_f32();

		
		if let Some(new_elem) = self.world.new_discovery {
			self.last_discovered = Some(new_elem);
			self.discovery_timer = 4.0; 
			self.world.new_discovery = None; 
		}

		
		if self.discovery_timer > 0.0 {
			self.discovery_timer -= 0.05;
			
			ctx.request_repaint(); 
		}

        ctx.input(|i| {
            

            let elements: Vec<Element> = Element::iter() 
				.filter(|e| !e.is_hidden() || self.world.discovered_elements.contains(e)) 
				.collect();

            
            let current_idx = elements
                .iter()
                .position(|&e| e == self.selected_element)
                .unwrap_or(0);

            if i.key_pressed(egui::Key::D) {
                let next_idx = (current_idx + 1) % elements.len();
                self.selected_element = elements[next_idx];
            }
            if i.key_pressed(egui::Key::A) {
                let next_idx = if current_idx == 0 {
                    elements.len() - 1
                } else {
                    current_idx - 1
                };
                self.selected_element = elements[next_idx];
            }

            if i.key_pressed(egui::Key::Space) {
                self.paused = !self.paused;
            }

            if i.key_pressed(egui::Key::E) {
                let powers: Vec<Power> = Power::iter().map(|(p, _, _)| p).collect();
                let current_power_idx = powers
                    .iter()
                    .position(|&p| p == self.selected_power)
                    .unwrap_or(0);
                let next_power_idx = (current_power_idx + 1) % powers.len();
                self.selected_power = powers[next_power_idx];
            }
            if i.key_pressed(egui::Key::Q) {
                let powers: Vec<Power> = Power::iter().map(|(p, _, _)| p).collect();
                let current_power_idx = powers
                    .iter()
                    .position(|&p| p == self.selected_power)
                    .unwrap_or(0);
                let next_power_idx = (current_power_idx - 1) % powers.len();
                self.selected_power = powers[next_power_idx];
            }

            let scroll = i.smooth_scroll_delta.y;
            self.scroll_accumulator += scroll;

            
            let threshold = 2.0; 

            if self.scroll_accumulator.abs() >= threshold {
                if self.scroll_accumulator > 0.0 {
                    self.brush_size = (self.brush_size + 1).min(20);
                } else {
                    self.brush_size = (self.brush_size - 1).max(1);
                }
                
                self.scroll_accumulator = 0.0;
            }
        });

        
        let panel = egui::Panel::top("top_panel");
        panel.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Dust — Simulation");
                ui.separator();
                ui.label(format!("FPS: {:.1}", self.fps));
                ui.separator();
                ui.checkbox(&mut self.paused, "Paused");
                ui.add(
                    egui::DragValue::new(&mut self.world.ambient_temp)
                        .speed(0.5)           
                        .range(-2000.0..=2000.0)
                        .suffix("°C")
                );
                ui.label("Ambient Temp");
                ui.separator();
                ui.add(egui::Slider::new(&mut self.game_speed, 1..=5).text("Game Speed"));
                ui.separator();
                ui.add(egui::Slider::new(&mut self.brush_size, 1..=20).text("Brush"));

                ui.separator();
                if ui.button("Screenshot").clicked() {
                    self.take_screenshot();
                }

                if ui.button("Clear").clicked() {
                    self.world = World::new(self.width, self.height);
					self.world.load_discoveries("discovered_elements.txt");
                }
            });
        });

        egui::TopBottomPanel::bottom("controls_panel")
            .resizable(false)
            .default_height(120.0)
            .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Elements");
                ui.horizontal_wrapped(|ui| {
                    let elements: Vec<Element> = Element::iter() 
					.filter(|e| {
						let starting_element = !e.is_hidden();
						let discovered = self.world.discovered_elements.contains(e);
						let not_secret = !e.is_super_hidden();
						
						(starting_element || discovered) && not_secret
					})
					.collect();

                    

                    for elem in elements {
                        let color = Color32::from_rgba_unmultiplied(
                            elem.color()[0],
                            elem.color()[1],
                            elem.color()[2],
                            elem.color()[3],
                        );
                        
                        
                        let text_color = if is_dark(color) {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        };

                        let label = elem.to_string();
                        
                        
                        let mut button = egui::Button::new(
                            egui::RichText::new(label)
                                .size(16.0)
                                .color(text_color), 
                        )
                        .fill(color)
                        .min_size(egui::vec2(80.0, 40.0));

                        
                        if self.selected_element == elem {
                            button = button.stroke(egui::Stroke::new(3.0, Color32::WHITE));
                        }

                        if ui.add(button).clicked() {
                            self.selected_element = elem;
                        }
                    }
				
                });

                let powers = Power::iter().collect::<Vec<_>>();
                ui.separator();
                ui.label("Powers");
                ui.horizontal_wrapped(|ui| {

                    for (power, label, color) in powers {
                        let button = if self.selected_power == power {
                            egui::Button::new(
                                egui::RichText::new(label).size(16.0).color(Color32::BLACK),
                            )
                            .fill(color)
                            .min_size(egui::vec2(80.0, 40.0))
                            .stroke(egui::Stroke::new(3.0, Color32::WHITE))
                        } else {
                            egui::Button::new(
                                egui::RichText::new(label).size(16.0).color(Color32::BLACK),
                            )
                            .fill(color)
                            .min_size(egui::vec2(80.0, 40.0))
                        };

                        if ui.add(button).clicked() {
                            self.selected_power = power;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Step").clicked() {
                        self.world.update();
                    }
                });
            }); 
        });

        egui::SidePanel::right("waila_panel")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("WAILA");
            ui.separator();

            
            if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                if self.sim_rect.contains(pointer_pos) {
                    let local_pos = pointer_pos - self.sim_rect.min;
                    let x = (local_pos.x / self.scale) as usize;
                    let y = (local_pos.y / self.scale) as usize;

                if let Some(info) = self.world.get_pixel_info(x as usize, y as usize) {
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("Element: {:?}", info.element)).strong());
                        ui.label(format!("Temperature: {:.1}°C", info.temp));
                        ui.label(format!("Age: {}", info.age));
                        ui.label(format!("Time Mod: {:.2}x", info.time_mod));
                        ui.label(format!("Conductivity: {:.2}x", info.conductivity));
                        ui.label(format!("Flammability: {:.2}", info.flammability));
                        ui.label(format!("Corrosion Resistance: {:.2}", info.corrosion_resistance));
						ui.label(format!("Density: {:.2}", info.density));
						ui.label(format!("Charge: {:.2}", info.charge));
                        ui.label("Reactions:");
                        if info.reactions.is_empty() {
                            ui.weak("None");
                        } else {
                            for reaction in info.reactions {
                                let conditions = reaction.conditions.iter().map(|cond| {
                                    match cond {
                                        world::element::Condition::RandomChance(p) => format!("Random Chance {:.2}%", p * 100.0),
                                        world::element::Condition::LifetimeGreater(t) => format!("Age > {} ticks", t),
                                        world::element::Condition::NearElement(e) => format!("Near {:?}", e),
                                        world::element::Condition::NotNearElement(e) => format!("Not Near {:?}", e),
										world::element::Condition::NearElementType(n) => format!("Near type {:?}", n),
                                        world::element::Condition::IsElementInRadius(e, r) => format!("Near {:?} within {}px", e, r),
                                        world::element::Condition::IsInsideOf(e) => format!("Inside {:?}", e),
                                        world::element::Condition::IsNotInsideOf(e) => format!("Not Inside {:?}", e),
                                        world::element::Condition::TemperatureAbove(t) => format!("Temp > {:.1}°C", t),
                                        world::element::Condition::TemperatureBelow(t) => format!("Temp < {:.1}°C", t),
                                        world::element::Condition::NearTemperatureAbove(t) => format!("Near Temp > {:.1}°C", t),
                                        world::element::Condition::NearTemperatureBelow(t) => format!("Near Temp < {:.1}°C", t),
                                        world::element::Condition::HasChargeAbove(c) => format!("Has Charge > {:.2}", c),
										world::element::Condition::HasChargeBelow(c) => format!("Has Charge < {:.2}", c),
                                    }
                                }).collect::<Vec<_>>().join("\n"); 
                                ui.label(egui::RichText::new(format!("If:\n{}", conditions)).italics().color(Color32::LIGHT_BLUE));
                                ui.label(egui::RichText::new(format!("-> Yields {:?}", reaction.output)).italics().color(Color32::LIGHT_BLUE));
                            }
                        }
                    });
                }
                } else {
                    ui.weak("Hover over the simulation");
                }
            } else {
                ui.weak("No input detected");
            }

            ui.separator();
            ui.label("Cursor World Pos:");
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                ui.monospace(format!("X: {:.0}, Y: {:.0}", pos.x / self.scale, pos.y / self.scale));
            }

			ui.separator();
			ui.label("Discovered Elements:");
			ui.label(format!("{}/{}", self.world.discovered_elements.len(), Element::COUNT - 2));
        });

            egui::CentralPanel::default().show(ctx, |ui| {
                self._ui(ui, _frame);
            });

            
            if !self.paused {
				if self.game_speed > 0 {
                	for _ in 0..self.game_speed {
                    	self.world.update();
                	}
					ctx.request_repaint();
				}
				else {
					for _ in 0..self.game_speed + 1{
						ctx.request_repaint();
					}
					self.world.update();
				}
                
            }
    }

    
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

    }

    
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(1400.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Dust",
        options,
        Box::new(|cc| Ok(Box::new(DustApp::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find canvas with id 'the_canvas_id'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element with id 'the_canvas_id' is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(DustApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}



fn noise_color(base: [u8; 4], shade: u8, seed: u32) -> [u8; 4] {
    let noise = signed_noise(shade, seed);
    let tint = match seed {
        1 => [12, 8, 0],
        2 => [0, 12, 20],
        3 => [8, 8, 8],
        _ => [0, 0, 0],
    };

    [
        clamp_color(base[0], tint[0], noise),
        clamp_color(base[1], tint[1], noise),
        clamp_color(base[2], tint[2], noise),
        base[3],
    ]
}

fn signed_noise(shade: u8, seed: u32) -> i16 {
    let mut value = shade as u32;
    value = value.wrapping_add(seed.wrapping_mul(668_265_263));
    value ^= value.wrapping_shl(13);
    value = value.wrapping_mul(1_274_126_177);
    let byte = ((value >> 16) & 0xFF) as i16;
    byte - 128
}

fn clamp_color(base: u8, tint: u8, noise: i16) -> u8 {
    let value = base as i16 + tint as i16 + (noise / 8);
    value.clamp(0, 255) as u8
}

fn is_dark(color: egui::Color32) -> bool {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    
    
    luminance < 0.45
}
