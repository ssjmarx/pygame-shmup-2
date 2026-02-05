use crate::command::Command;
use crate::object_pool::ObjectPool;
use crate::star::Star;

/// Movement mode enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementMode {
    Normal,    // Both engines, 1200 px/s
    Control,    // Thrusters only, 600 px/s, mouse aim
    Boost,      // Engine only, 2400 px/s
    Alt,        // No resistance, unlimited speed
    Disabled,   // Both engines disabled (Ctrl+Shift)
}

/// Player entity
#[derive(Debug, Clone)]
pub struct Player {
    // Position in world coordinates
    pub x: f64,
    pub y: f64,

    // Velocity in units/second
    pub vx: f64,
    pub vy: f64,

    // Rotation in radians
    pub rotation: f64,  // This is now "facing_angle"
    
    // New fields for enhanced movement
    pub facing_angle: f64,              // Ship's facing direction (independent of velocity)
    pub engine_spool_time: f64,         // How long main engine has been spooling (0.0 to 1.0)
    pub input_dx: f64,                  // Normalized input direction X (-1.0 to 1.0)
    pub input_dy: f64,                  // Normalized input direction Y (-1.0 to 1.0)
    pub mouse_target_angle: Option<f64>, // Target angle for control/alt mode
    
    // Mode flags
    pub control_mode: bool,              // Ctrl held (precision mode)
    pub boost_mode: bool,               // Shift held
    pub alt_mode: bool,                 // Alt held (no resistance)
    
    // Physics constants
    pub rotation_speed: f64,
    pub thruster_acceleration: f64,
    pub main_engine_acceleration: f64,
    pub boost_engine_acceleration: f64,  // Explicit boost acceleration
    pub min_resistance: f64,
    pub resistance_max: f64,  // Maximum resistance for above-speed deceleration
    pub engine_spool_rate: f64,
    pub engine_spool_min_factor: f64,  // Minimum spool factor (0.5)
    pub engine_spool_max_factor: f64,  // Maximum spool factor (1.0)
    
    // Resistance thresholds (multipliers of top speed)
    pub resistance_threshold_low: f64,   // Below this: ramp from min to top_speed
    pub resistance_threshold_high: f64,  // Above this: use resistance_max
    
    // Size for rendering
    pub width: f64,
    pub height: f64,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            rotation: -std::f64::consts::PI / 2.0,  // Point up
            facing_angle: -std::f64::consts::PI / 2.0,  // Point up initially
            engine_spool_time: 0.0,
            input_dx: 0.0,
            input_dy: 0.0,
            mouse_target_angle: None,
            control_mode: false,
            boost_mode: false,
            alt_mode: false,
            rotation_speed: 4.0,              // ~229 degrees/second
            thruster_acceleration: 1000.0,     // Maneuvering thrusters
            main_engine_acceleration: 600.0,   // Normal main engine thrust
            boost_engine_acceleration: 1200.0,  // Boost mode acceleration (was derived 600*2)
            min_resistance: 100.0,            // Resistance floor
            resistance_max: 2400.0,           // Maximum resistance for rapid deceleration
            engine_spool_rate: 1.0,           // Time to reach full spool (seconds)
            engine_spool_min_factor: 0.5,       // Minimum engine power when starting (50%)
            engine_spool_max_factor: 1.0,       // Maximum engine power at full spool (100%)
            
            // Resistance scaling thresholds (multipliers of top speed)
            resistance_threshold_low: 1.0,      // Below 1x: ramp from min to top_speed resistance
            resistance_threshold_high: 2.0,     // Above 2x: use resistance_max
            
            width: 15.0,
            height: 20.0,
        }
    }
    
    /// Get current movement mode
    pub fn get_current_mode(&self) -> MovementMode {
        if self.control_mode && self.boost_mode {
            MovementMode::Disabled  // Ctrl+Shift: both engines disabled
        } else if self.alt_mode {
            MovementMode::Alt  // Alt: no resistance, engines enabled based on other modes
        } else if self.control_mode {
            MovementMode::Control  // Ctrl: precision mode, thrusters only, mouse aim
        } else if self.boost_mode {
            MovementMode::Boost  // Shift: boost mode, main engine only, 2x speed
        } else {
            MovementMode::Normal  // Default: both engines, normal speed
        }
    }
    
    /// Get top speed for current mode
    pub fn get_top_speed(&self) -> f64 {
        match self.get_current_mode() {
            MovementMode::Normal => 400.0,   // Current settings give ~ 615 top speed
            MovementMode::Control => 200.0,  // Current settings give ~ 286 top speed
            MovementMode::Boost => 1200.0,  // Current settings give ~ 1181 top speed
            MovementMode::Alt => f64::INFINITY,  // Alt mode: unlimited speed
            MovementMode::Disabled => 0.0,
        }
    }
    
    /// Get main engine acceleration (explicit boost value)
    pub fn get_main_engine_acceleration(&self) -> f64 {
        // Boost mode uses explicit boost acceleration value
        if self.boost_mode {
            self.boost_engine_acceleration
        } else {
            self.main_engine_acceleration
        }
    }
    
    /// Check if thrusters are enabled in current mode
    pub fn thrusters_enabled(&self) -> bool {
        match self.get_current_mode() {
            MovementMode::Normal => true,
            MovementMode::Control => true,
            MovementMode::Boost => false,  // Boost disables thrusters
            MovementMode::Alt => !self.boost_mode,  // Alt+Boost: no thrusters
            MovementMode::Disabled => false,
        }
    }
    
    /// Check if main engine is enabled in current mode
    pub fn main_engine_enabled(&self) -> bool {
        match self.get_current_mode() {
            MovementMode::Normal => true,
            MovementMode::Control => false,  // Control disables main engine
            MovementMode::Boost => true,
            MovementMode::Alt => !self.control_mode,  // Alt+Control: no main engine
            MovementMode::Disabled => false,
        }
    }
    
    /// Check if resistance is enabled in current mode
    pub fn resistance_enabled(&self) -> bool {
        !self.alt_mode  // Alt mode disables resistance
    }
    
    /// Apply maneuvering thrusters
    fn apply_thrusters(&mut self, dt: f64) {
        if self.input_dx.abs() < 0.01 && self.input_dy.abs() < 0.01 {
            return; // No input, no thrust
        }
        
        // Apply instant full-power acceleration in input direction
        self.vx += self.input_dx * self.thruster_acceleration * dt;
        self.vy += self.input_dy * self.thruster_acceleration * dt;
    }
    
    /// Apply main engine with spool-up mechanics
    fn apply_main_engine(&mut self, dt: f64) {
        if self.input_dx.abs() < 0.01 && self.input_dy.abs() < 0.01 {
            self.engine_spool_time = 0.0;
            return; // No input, no engine
        }
        
        // Update spool time (max at ENGINE_SPOOL_RATE)
        self.engine_spool_time = (self.engine_spool_time + dt).min(self.engine_spool_rate);
        
        // Calculate spool factor with explicit min/max values
        let spool_progress = (self.engine_spool_time / self.engine_spool_rate).min(1.0);
        let spool_factor = self.engine_spool_min_factor + 
            (self.engine_spool_max_factor - self.engine_spool_min_factor) * spool_progress;
        
        // Get current main engine acceleration (2x in boost mode)
        let base_accel = self.get_main_engine_acceleration();
        
        // Apply acceleration in facing direction with spool factor
        let engine_accel = base_accel * spool_factor;
        let ax = self.facing_angle.cos() * engine_accel;
        let ay = self.facing_angle.sin() * engine_accel;
        
        self.vx += ax * dt;
        self.vy += ay * dt;
    }
    
    /// Apply resistance that scales with speed
    fn apply_resistance(&mut self, dt: f64) {
        let top_speed = self.get_top_speed();
        if top_speed == 0.0 || top_speed == f64::INFINITY {
            return;
        }
        
        // Calculate current speed
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        
        if speed < 1.0 {
            return; // Already stopped
        }
        
        // Calculate resistance magnitude with explicit thresholds:
        // - At 0 speed: min_resistance (ensures complete stop)
        // - At threshold_low * top_speed: resistance = top_speed (allows maintaining speed)
        // - At threshold_high * top_speed: resistance = resistance_max (rapid deceleration)
        // - Above threshold_high: clamp at resistance_max
        
        let speed_ratio = speed / top_speed;  // 0.0 to infinity
        
        let resistance = if speed_ratio <= self.resistance_threshold_low {
            // Below low threshold: ramp from min_resistance to top_speed
            let t = speed_ratio / self.resistance_threshold_low;  // 0.0 to 1.0
            top_speed.min(self.min_resistance + (top_speed - self.min_resistance) * t)
        } else if speed_ratio <= self.resistance_threshold_high {
            // Between low and high thresholds: interpolate from top_speed to resistance_max
            let t = (speed_ratio - self.resistance_threshold_low) / 
                (self.resistance_threshold_high - self.resistance_threshold_low);  // 0.0 to 1.0
            top_speed + t * (self.resistance_max - top_speed)
        } else {
            // Above high threshold: clamp at resistance_max
            self.resistance_max
        };
        
        // Apply as acceleration opposite to velocity
        let vx_norm = self.vx / speed;
        let vy_norm = self.vy / speed;
        
        self.vx -= vx_norm * resistance * dt;
        self.vy -= vy_norm * resistance * dt;
    }
    
    /// Apply rotation toward target
    fn apply_rotation(&mut self, dt: f64) {
        let target_angle = match self.get_current_mode() {
            MovementMode::Control => {
                // Control mode: rotate toward mouse
                self.mouse_target_angle.unwrap_or(self.facing_angle)
            }
            MovementMode::Alt => {
                // Alt mode: rotate toward input direction (not mouse!)
                if self.input_dx.abs() > 0.01 || self.input_dy.abs() > 0.01 {
                    self.input_dy.atan2(self.input_dx)
                } else {
                    self.facing_angle // Keep current facing if no input
                }
            }
            MovementMode::Normal | MovementMode::Boost => {
                // Normal and Boost modes: rotate toward input direction
                if self.input_dx.abs() > 0.01 || self.input_dy.abs() > 0.01 {
                    self.input_dy.atan2(self.input_dx)
                } else {
                    self.facing_angle // Keep current facing if no input
                }
            }
            MovementMode::Disabled => self.facing_angle,
        };
        
        // Smooth rotation toward target
        let angle_diff = shortest_angle_diff(self.facing_angle, target_angle);
        let max_rotation = self.rotation_speed * dt;
        
        if angle_diff.abs() <= max_rotation {
            self.facing_angle = target_angle;
        } else {
            self.facing_angle += angle_diff.signum() * max_rotation;
        }
        
        // Update rotation field for rendering (add 90 degrees clockwise = PI/2)
        self.rotation = self.facing_angle + std::f64::consts::PI / 2.0;
    }
    
    
    /// Main update method
    pub fn update(&mut self, dt: f64) {
        // 1. Apply maneuvering thrusters (if active)
        if self.thrusters_enabled() {
            self.apply_thrusters(dt);
        }
        
        // 2. Apply main engine (if active, with spool-up)
        if self.main_engine_enabled() {
            self.apply_main_engine(dt);
        } else {
            // Reset spool when main engine inactive
            self.engine_spool_time = 0.0;
        }
        
        // 3. Apply resistance (if enabled)
        if self.resistance_enabled() {
            self.apply_resistance(dt);
        }
        
        // 4. Apply rotation
        self.apply_rotation(dt);
        
        // 5. Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

/// Calculate shortest angle difference between two angles
fn shortest_angle_diff(from: f64, to: f64) -> f64 {
    let mut diff = to - from;
    while diff > std::f64::consts::PI {
        diff -= 2.0 * std::f64::consts::PI;
    }
    while diff < -std::f64::consts::PI {
        diff += 2.0 * std::f64::consts::PI;
    }
    diff
}

/// Game state container
pub struct GameState {
    player: Player,
    command_buffer: Vec<Command>,
    stars: ObjectPool<Star>,
    prev_camera_x: f64,
    prev_camera_y: f64,
    camera_x: f64,  // Actual camera position
    camera_y: f64,  // Actual camera position
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
        let initial_cam_x = player.x - 400.0;
        let initial_cam_y = player.y - 300.0;
        
        GameState {
            player,
            command_buffer: Vec::new(),
            stars,
            prev_camera_x: initial_cam_x,
            prev_camera_y: initial_cam_y,
            camera_x: initial_cam_x,
            camera_y: initial_cam_y,
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
            }
        }
        
        // Normalize input direction for diagonal movement
        let input_mag = (self.player.input_dx * self.player.input_dx + 
                         self.player.input_dy * self.player.input_dy).sqrt();
        if input_mag > 0.01 {
            self.player.input_dx /= input_mag;
            self.player.input_dy /= input_mag;
        }
        
        // Update player physics
        self.player.update(dt);
        
        // Update stars with parallax
        // Use smoothed camera position (same as Camera struct logic)
        let target_cam_x = self.player.x - 400.0;  // Center player on screen
        let target_cam_y = self.player.y - 300.0;
        
        // Apply smoothing (0.4 lerp factor, same as Camera struct)
        let smoothing = 0.4;
        self.camera_x += (target_cam_x - self.camera_x) * smoothing;
        self.camera_y += (target_cam_y - self.camera_y) * smoothing;
        
        // Calculate camera movement delta
        let cam_dx = self.camera_x - self.prev_camera_x;
        let cam_dy = self.camera_y - self.prev_camera_y;
        
        // Update stars with parallax
        let mut to_respawn: Vec<(usize, Star)> = Vec::new();
        
        for (index, star) in self.stars.iter_active_mut() {
            // Apply parallax: move star by (camera movement * depth * 0.25)
            // Reduced parallax effect to 25% of camera movement
            // Depth 0.1 = barely moves (far), Depth 1.0 = moves 25% with camera (close)
            star.x += cam_dx * star.depth * 0.25;
            star.y += cam_dy * star.depth * 0.25;
            
            // Update twinkle
            star.update(dt, self.player.x, self.player.y, self.camera_x, self.camera_y);
            
            // Respawn stars that are too far from screen
            let screen_x = star.x - self.camera_x;
            let screen_y = star.y - self.camera_y;
            
            // If star is way off screen, mark for respawn
            if screen_x < -200.0 || screen_x > 1000.0 || 
               screen_y < -200.0 || screen_y > 800.0 {
                
                let new_star = Star::new_random_at_edge(self.camera_x, self.camera_y);
                to_respawn.push((index, new_star));
            }
        }
        
        // Respawn marked stars (after iteration to avoid borrow issues)
        for (index, new_star) in to_respawn {
            self.stars.deallocate(index);
            self.stars.allocate(new_star);
        }
        
        // Update previous camera position
        self.prev_camera_x = self.camera_x;
        self.prev_camera_y = self.camera_y;
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }
    
    pub fn get_star_render_data(&self) -> Vec<crate::star::StarRenderData> {
        self.stars.iter_active()
            .map(|(_, star)| star.to_render_data(self.camera_x, self.camera_y))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_detection_normal() {
        let player = Player::new();
        assert_eq!(player.get_current_mode(), MovementMode::Normal);
        assert_eq!(player.get_top_speed(), 400.0);  // Normal mode top speed
        assert!(player.thrusters_enabled());
        assert!(player.main_engine_enabled());
        assert!(player.resistance_enabled());
    }

    #[test]
    fn test_mode_detection_control() {
        let mut player = Player::new();
        player.control_mode = true;
        assert_eq!(player.get_current_mode(), MovementMode::Control);
        assert_eq!(player.get_top_speed(), 200.0);  // Control mode top speed
        assert!(player.thrusters_enabled());
        assert!(!player.main_engine_enabled());
        assert!(player.resistance_enabled());
    }

    #[test]
    fn test_mode_detection_boost() {
        let mut player = Player::new();
        player.boost_mode = true;
        assert_eq!(player.get_current_mode(), MovementMode::Boost);
        assert_eq!(player.get_top_speed(), 1200.0);  // Halved from 2400
        assert!(!player.thrusters_enabled());
        assert!(player.main_engine_enabled());
        assert!(player.resistance_enabled());
    }

    #[test]
    fn test_mode_detection_alt() {
        let mut player = Player::new();
        player.alt_mode = true;
        assert_eq!(player.get_current_mode(), MovementMode::Alt);
        assert_eq!(player.get_top_speed(), f64::INFINITY);
        assert!(player.thrusters_enabled());
        assert!(player.main_engine_enabled());
        assert!(!player.resistance_enabled());
    }

    #[test]
    fn test_mode_detection_disabled() {
        let mut player = Player::new();
        player.control_mode = true;
        player.boost_mode = true;
        assert_eq!(player.get_current_mode(), MovementMode::Disabled);
        assert_eq!(player.get_top_speed(), 0.0);
        assert!(!player.thrusters_enabled());
        assert!(!player.main_engine_enabled());
    }

    #[test]
    fn test_spool_up_reaches_max_in_one_second() {
        let mut player = Player::new();
        player.input_dx = 1.0;
        
        // Simulate 1 second at 60 FPS (62.5 frames to account for timing)
        for _ in 0..63 {
            player.apply_main_engine(0.016);
        }
        
        // Spool time should be at maximum (1.0)
        assert!((player.engine_spool_time - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spool_down_resets_properly() {
        let mut player = Player::new();
        player.input_dx = 1.0;
        
        // Spool up
        for _ in 0..60 {
            player.apply_main_engine(0.016);
        }
        assert!(player.engine_spool_time > 0.9);
        
        // Reset input
        player.input_dx = 0.0;
        player.apply_main_engine(0.016);
        
        // Spool should be reset
        assert!(player.engine_spool_time < 0.01);
    }

    #[test]
    fn test_thrusters_provide_immediate_acceleration() {
        let mut player = Player::new();
        player.input_dx = 1.0;
        player.input_dy = 0.0;
        
        // Apply thrusters for one frame
        player.apply_thrusters(0.016);
        
        // Should have immediate velocity
        assert!(player.vx > 0.0);
        // 1000 px/s² * 0.016s = 16 px/s
        assert!((player.vx - 16.0).abs() < 0.5);
    }

    #[test]
    fn test_thrusters_disabled_in_boost_mode() {
        let mut player = Player::new();
        player.boost_mode = true;
        player.input_dx = 1.0;
        
        assert!(!player.thrusters_enabled());
    }

    #[test]
    fn test_boost_mode_doubles_acceleration() {
        let mut player = Player::new();
        player.boost_mode = true;
        
        assert_eq!(player.get_main_engine_acceleration(), 1200.0);  // Explicit boost acceleration
    }

    #[test]
    fn test_boost_mode_doubles_top_speed() {
        let mut player = Player::new();
        player.boost_mode = true;
        
        assert_eq!(player.get_top_speed(), 1200.0);  // Halved from 2400
    }

    #[test]
    fn test_alt_mode_disables_resistance() {
        let mut player = Player::new();
        player.alt_mode = true;
        
        assert!(!player.resistance_enabled());
        assert_eq!(player.get_top_speed(), f64::INFINITY);
    }

    #[test]
    fn test_rotation_smooth_interpolation() {
        let mut player = Player::new();
        player.facing_angle = 0.0;  // Start facing right (0 radians)
        player.input_dx = 1.0;      // Input is also right
        player.input_dy = 0.0;
        
        // Apply rotation for one frame
        player.apply_rotation(0.016);
        
        // Target angle is atan2(0, 1) = 0 (right)
        // Rotation speed is 1 rad/s, so should rotate toward 0
        // Already at 0, so should stay at 0
        assert!((player.facing_angle - 0.0).abs() < 0.01);
    }

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
    fn test_shortest_angle_diff() {
        // Test wrapping around PI
        let diff = shortest_angle_diff(3.0, -3.0);
        // Should wrap to ~-0.283, not 6.0
        assert!(diff.abs() < 1.0);
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

    #[test]
    fn test_control_mode_mouse_aiming() {
        let mut player = Player::new();
        player.control_mode = true;
        
        // Set mouse target to angle 1.0
        player.mouse_target_angle = Some(1.0);
        
        // Update rotation (should rotate toward 1.0 from starting -PI/2)
        player.apply_rotation(0.1);
        
        // With dt=0.1 and rotation_speed=4.0, should rotate 0.4 radians
        // Starting from -PI/2 (-1.571), should rotate toward 1.0
        // The shortest angle from -1.571 to 1.0 is going forward through 0
        // So should rotate: -1.571 + 0.4 = -1.171
        assert!(player.facing_angle < -1.0);  // Should have moved toward target
        assert!(player.facing_angle > -1.6);  // But not too far
    }

    #[test]
    fn test_alt_boost_mode() {
        let mut player = Player::new();
        player.alt_mode = true;
        player.boost_mode = true;
        
        assert_eq!(player.get_current_mode(), MovementMode::Alt);
        assert!(!player.thrusters_enabled());  // Boost disables thrusters
        assert!(player.main_engine_enabled());   // Alt keeps main engine
        assert_eq!(player.get_main_engine_acceleration(), 1200.0);  // Explicit boost acceleration
        assert!(!player.resistance_enabled());  // Alt disables resistance
    }

    #[test]
    fn test_alt_control_mode() {
        let mut player = Player::new();
        player.alt_mode = true;
        player.control_mode = true;
        
        assert_eq!(player.get_current_mode(), MovementMode::Alt);
        assert!(player.thrusters_enabled());    // Control enables thrusters
        assert!(!player.main_engine_enabled()); // Control disables main engine
        assert!(!player.resistance_enabled());  // Alt disables resistance
    }
}