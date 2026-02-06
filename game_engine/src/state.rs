use crate::command::Command;
use crate::config::{GunConfig, ProjectileConfig, PlayerGunConfig};
use crate::camera::Camera;
use crate::gun::{FireSector, Gun, get_fire_sector};
use crate::object_pool::ObjectPool;
use crate::player::Player;
use crate::projectile::{Projectile, ProjectileRenderData, ProjectileType};
use crate::star::Star;

/// Game state container
pub struct GameState {
    player: Player,
    command_buffer: Vec<Command>,
    stars: ObjectPool<Star>,
    projectiles: ObjectPool<Projectile>,
    camera: Camera,
    current_time: f64,
    prev_control_mode: bool,  // Track control mode transitions
    was_autofiring: bool,      // Track if autofire was active (for tracking shot prevention)
    autofire_start_time: f64,  // Track when autofire started (for 0.5s delay)
}

impl GameState {
    pub fn new() -> Self {
        let mut stars = ObjectPool::new(125, Star::default());
        
        // Populate pool with 125 random stars around initial position
        // Wider range than screen to give stars to discover as player moves
        for _ in 0..125 {
            let star = Star::new_random_in_screen(
                (-400.0, 1200.0),  // Wider X range (800 screen + 400 margin each side)
                (-400.0, 1200.0)   // Wider Y range (600 screen + 400 margin each side)
            );
            stars.allocate(star);  // Always succeeds with 125 capacity
        }
        
        let player = Player::new();
        
        GameState {
            player,
            command_buffer: Vec::new(),
            stars,
            projectiles: ObjectPool::new(100, Projectile::default()),
            camera: Camera::new(0.0, 0.0, 800.0, 600.0),
            current_time: 0.0,
            prev_control_mode: false,
            was_autofiring: false,
            autofire_start_time: 0.0,
        }
    }

    pub fn add_command(&mut self, command: Command) {
        self.command_buffer.push(command);
    }

    pub fn update(&mut self, dt: f64) {
        // Reset input direction
        self.player.input_dx = 0.0;
        self.player.input_dy = 0.0;
        
        // Process commands
        for cmd in self.command_buffer.drain(..) {
            match cmd {
                Command::MoveUp => self.player.input_dy = -1.0,
                Command::MoveDown => self.player.input_dy = 1.0,
                Command::MoveLeft => self.player.input_dx = -1.0,
                Command::MoveRight => self.player.input_dx = 1.0,
                Command::ToggleAltMode(state) => self.player.alt_mode = state,
                Command::ToggleBoostMode(state) => self.player.boost_mode = state,
                Command::ToggleControlMode(state) => self.player.control_mode = state,
                Command::SetMouseTarget(x, y, cam_x, cam_y) => {
                    // Use provided camera position from Command
                    let world_x = x + cam_x;
                    let world_y = y + cam_y;
                    
                    // Calculate angle from player to mouse
                    let dx = world_x - self.player.x;
                    let dy = world_y - self.player.y;
                    
                    self.player.mouse_target_angle = Some(dy.atan2(dx));
                }
                Command::SetTargetEntity(entity_id) => {
                    self.player.mouse_target_entity_id = entity_id;
                }
                Command::StartShootingTracking => {
                    // Prevent tracking shot if was autofiring (only fire on brief clicks)
                    if self.was_autofiring {
                        self.was_autofiring = false;  // Reset flag
                        // Don't fire tracking shot
                    } else {
                        // Check cooldown BEFORE firing
                        let time_since_last = self.current_time - self.player.last_tracking_shot_time;
                        
                        if time_since_last >= self.player.gun_config.tracking_cooldown {
                            self.player.last_tracking_shot_time = self.current_time;
                            
                            // Calculate fire sector based on mouse position relative to player
                            let sector = get_fire_sector(
                                self.player.mouse_target_angle.unwrap_or(self.player.facing_angle),
                                self.player.facing_angle
                            );
                            
                            let config = ProjectileConfig::default();
                            
                            // Left gun fires only in Left or Both sectors
                            if sector == FireSector::Left || sector == FireSector::Both {
                                let left_angle = self.player.left_gun.get_firing_angle();
                                let left_rx = self.player.left_gun.offset_x * self.player.rotation.cos() - self.player.left_gun.offset_y * self.player.rotation.sin();
                                let left_ry = self.player.left_gun.offset_x * self.player.rotation.sin() + self.player.left_gun.offset_y * self.player.rotation.cos();
                                let left_x = self.player.x + left_rx;
                                let left_y = self.player.y + left_ry;
                                
                                if let Some(_) = self.projectiles.allocate(Projectile::new_tracking(
                                    left_x, left_y, left_angle,
                                    None,  // Don't target player
                                    config.clone(),
                                    self.player.vx,
                                    self.player.vy
                                )) {
                                    self.player.left_gun.add_recoil(config.tracking_recoil_amount);
                                }
                            }
                            
                            // Right gun fires only in Right or Both sectors
                            if sector == FireSector::Right || sector == FireSector::Both {
                                let right_angle = self.player.right_gun.get_firing_angle();
                                let right_rx = self.player.right_gun.offset_x * self.player.rotation.cos() - self.player.right_gun.offset_y * self.player.rotation.sin();
                                let right_ry = self.player.right_gun.offset_x * self.player.rotation.sin() + self.player.right_gun.offset_y * self.player.rotation.cos();
                                let right_x = self.player.x + right_rx;
                                let right_y = self.player.y + right_ry;
                                
                                if let Some(_) = self.projectiles.allocate(Projectile::new_tracking(
                                    right_x, right_y, right_angle,
                                    None,  // Don't target player
                                    config.clone(),
                                    self.player.vx,
                                    self.player.vy
                                )) {
                                    self.player.right_gun.add_recoil(config.tracking_recoil_amount);
                                }
                            }
                        }
                    }
                    self.was_autofiring = false;  // Reset flag
                }
                Command::StopShootingTracking => {
                    // No longer tracking firing state (removed)
                }
                Command::StartAutoFire => {
                    self.player.is_autofiring = true;
                    self.autofire_start_time = self.current_time;  // Record when autofire started
                }
                Command::StopAutoFire => {
                    self.player.is_autofiring = false;
                    // Don't reset was_autofiring here - let StartShootingTracking handle it
                }
            }
        }
        
        // Normalize input direction for diagonal movement
        let input_mag = (self.player.input_dx * self.player.input_dx + 
                         self.player.input_dy * self.player.input_dy).sqrt();
        if input_mag > 0.01 {
            self.player.input_dx /= input_mag;
            self.player.input_dy /= input_mag;
        }
        
        // Update time BEFORE player movement (needed by autofire timing)
        self.current_time += dt;
        
        // Update guns tracking BEFORE player movement (so spawn points use pre-movement position)
        // Set gun target angle to mouse target (same for both guns)
        if let Some(mouse_angle) = self.player.mouse_target_angle {
            self.player.left_gun.set_target_angle(mouse_angle);
            self.player.right_gun.set_target_angle(mouse_angle);
        }
        
        // Handle autofire (hold) BEFORE player movement
        // This ensures spawn points use pre-movement player position
        if self.player.is_autofiring {
            // Check if 0.5s delay has passed
            let autofire_active = self.current_time - self.autofire_start_time >= 0.5;
            
            // Spool up autofire rate immediately when holding (not after delay)
            self.player.left_gun.spool_up_autofire(dt);
            self.player.right_gun.spool_up_autofire(dt);
            
            // Only fire bullets after 0.5s delay has passed
            if autofire_active {
                self.was_autofiring = true;
                
                // Fire bullets
                let sector = get_fire_sector(
                    self.player.mouse_target_angle.unwrap_or(self.player.facing_angle),
                    self.player.facing_angle
                );
                
                let config = ProjectileConfig::default();
                
                // Left gun fires only in Left or Both sectors
                if sector == FireSector::Left || sector == FireSector::Both {
                    if self.player.left_gun.update_autofire(self.current_time) {
                        let left_angle = self.player.left_gun.get_firing_angle();
                        let left_rx = self.player.left_gun.offset_x * self.player.rotation.cos() - self.player.left_gun.offset_y * self.player.rotation.sin();
                        let left_ry = self.player.left_gun.offset_x * self.player.rotation.sin() + self.player.left_gun.offset_y * self.player.rotation.cos();
                        let left_x = self.player.x + left_rx;
                        let left_y = self.player.y + left_ry;
                        
                        self.projectiles.allocate(Projectile::new_autofire(left_x, left_y, left_angle, config.clone(),
                                                                     self.player.vx, self.player.vy));
                        self.player.left_gun.add_recoil(config.autofire_recoil_amount);
                    }
                }
                
                // Right gun fires only in Right or Both sectors
                if sector == FireSector::Right || sector == FireSector::Both {
                    if self.player.right_gun.update_autofire(self.current_time) {
                        let right_angle = self.player.right_gun.get_firing_angle();
                        let right_rx = self.player.right_gun.offset_x * self.player.rotation.cos() - self.player.right_gun.offset_y * self.player.rotation.sin();
                        let right_ry = self.player.right_gun.offset_x * self.player.rotation.sin() + self.player.right_gun.offset_y * self.player.rotation.cos();
                        let right_x = self.player.x + right_rx;
                        let right_y = self.player.y + right_ry;
                        
                        self.projectiles.allocate(Projectile::new_autofire(right_x, right_y, right_angle, config.clone(),
                                                                       self.player.vx, self.player.vy));
                        self.player.right_gun.add_recoil(config.autofire_recoil_amount);
                    }
                }
            }
        }
        
        // Update both guns - pass player rotation so guns can rotate their base angle
        // This happens EVERY frame for tracking and spool down
        self.player.left_gun.update_tracking_with_ship(
            self.player.facing_angle,
            self.player.rotation,  // Ship's visual rotation
            self.player.vx,
            self.player.vy,
            self.player.get_top_speed(),
            self.player.gun_config.movement_compensation,
            dt
        );
        
        self.player.right_gun.update_tracking_with_ship(
            self.player.facing_angle,
            self.player.rotation,  // Ship's visual rotation
            self.player.vx,
            self.player.vy,
            self.player.get_top_speed(),
            self.player.gun_config.movement_compensation,
            dt
        );
        
        // Update player physics
        self.player.update(dt);
        
        // Update projectiles
        // Collect entity positions for tracking (just player for now)
        let entities = [(self.player.x, self.player.y, 999)]; // Player ID 999
        let mut to_remove: Vec<usize> = Vec::new();
        
        for (index, proj) in self.projectiles.iter_active_mut() {
            proj.update(dt, &entities);
            
            if proj.is_expired() {
                to_remove.push(index);
            }
        }
        
        // Remove expired projectiles
        for index in to_remove {
            self.projectiles.deallocate(index);
        }
        
        // Update camera and get movement delta for parallax
        let (cam_dx, cam_dy) = self.camera.update(&self.player, dt);
        
        // Update stars with parallax
        let mut to_respawn: Vec<(usize, Star)> = Vec::new();
        
        for (index, star) in self.stars.iter_active_mut() {
            // Apply parallax: move star by (camera movement * depth * 0.25)
            // Reduced parallax effect to 25% of camera movement
            // Depth 0.1 = barely moves (far), Depth 1.0 = moves 25% with camera (close)
            star.x += cam_dx * star.depth * 0.25;
            star.y += cam_dy * star.depth * 0.25;
            
            // Update twinkle
            star.update(dt, self.player.x, self.player.y, self.camera.get_x(), self.camera.get_y());
            
            // Respawn stars that are too far from screen
            let screen_x = star.x - self.camera.get_x();
            let screen_y = star.y - self.camera.get_y();
            
            // If star is way off screen, mark for respawn
            if screen_x < -200.0 || screen_x > 1000.0 || 
               screen_y < -200.0 || screen_y > 800.0 {
                
                let new_star = Star::new_random_at_edge(self.camera.get_x(), self.camera.get_y());
                to_respawn.push((index, new_star));
            }
        }
        
        // Respawn marked stars (after iteration to avoid borrow issues)
        for (index, new_star) in to_respawn {
            self.stars.deallocate(index);
            self.stars.allocate(new_star);
        }
        
        // Update previous control mode state
        self.prev_control_mode = self.player.control_mode;
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }
    
    pub fn get_star_render_data(&self) -> Vec<crate::star::StarRenderData> {
        self.stars.iter_active()
            .map(|(_, star)| star.to_render_data(self.camera.get_x(), self.camera.get_y()))
            .collect()
    }
    
    pub fn get_projectile_render_data(&self) -> Vec<ProjectileRenderData> {
        self.projectiles.iter_active()
            .map(|(_, proj)| {
                let screen_x = proj.x - self.camera.get_x();
                let screen_y = proj.y - self.camera.get_y();
                
                ProjectileRenderData {
                    x: screen_x,
                    y: screen_y,
                    rotation: proj.get_rotation(),
                    length: proj.length,
                    width: proj.size,
                    color: proj.get_color(),
                }
            })
            .collect()
    }
    
    pub fn get_gun_angles(&self) -> (f64, f64) {
        (self.player.left_gun.angle, self.player.right_gun.angle)
    }
    
    pub fn get_camera_x(&self) -> f64 {
        self.camera.get_x()
    }
    
    pub fn get_camera_y(&self) -> f64 {
        self.camera.get_y()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagonal_input_normalized() {
        let mut state = GameState::new();
        state.add_command(Command::MoveUp);
        state.add_command(Command::MoveRight);
        
        // Simulate one update
        for cmd in state.command_buffer.drain(..) {
            match cmd {
                Command::MoveUp => state.player.input_dy = -1.0,
                Command::MoveRight => state.player.input_dx = 1.0,
                _ => {}
            }
        }
        
        // Normalize
        let input_mag = (state.player.input_dx * state.player.input_dx + 
                         state.player.input_dy * state.player.input_dy).sqrt();
        if input_mag > 0.01 {
            state.player.input_dx /= input_mag;
            state.player.input_dy /= input_mag;
        }
        
        // Should be normalized to ~0.707
        assert!((state.player.input_dx.abs() - 0.707).abs() < 0.01);
        assert!((state.player.input_dy.abs() - 0.707).abs() < 0.01);
    }

    #[test]
    fn test_normal_mode_movement() {
        let mut state = GameState::new();
        
        // Simulate 30 frames moving right
        for _ in 0..30 {
            state.add_command(Command::MoveRight);
            state.update(0.016);
        }
        
        // Should have moved right (X position should be positive)
        assert!(state.player.x > 50.0);
        // Y position may not be exactly 0 due to rotation during movement
        assert!(state.player.y.abs() < 50.0);  // But should be relatively small
    }
}