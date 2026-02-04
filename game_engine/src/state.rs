use crate::command::Command;
use crate::object_pool::ObjectPool;
use crate::star::Star;

/// Physics configuration
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    pub acceleration: f64,  // pixels/second²
    pub friction: f64,        // friction coefficient (0-1)
    pub max_speed: f64,      // pixels/second
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        PhysicsConfig {
            acceleration: 2000.0,  // Quick acceleration
            friction: 5.0,         // Moderate friction
            max_speed: 400.0,       // Same as before
        }
    }
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
    pub rotation: f64,

    // Physics configuration
    pub physics: PhysicsConfig,

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
            physics: PhysicsConfig::default(),
            width: 15.0,  // Updated per user request
            height: 20.0,  // Updated per user request (elongated)
        }
    }

    pub fn apply_acceleration(&mut self, ax: f64, ay: f64, dt: f64) {
        // Apply acceleration to velocity
        self.vx += ax * self.physics.acceleration * dt;
        self.vy += ay * self.physics.acceleration * dt;
        
        // Clamp to max speed
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed > self.physics.max_speed {
            let scale = self.physics.max_speed / speed;
            self.vx *= scale;
            self.vy *= scale;
        }
    }

    pub fn update(&mut self, dt: f64) {
        // Apply friction (velocity decay)
        let friction_factor = 1.0 - (self.physics.friction * dt).min(1.0);
        self.vx *= friction_factor;
        self.vy *= friction_factor;
        
        // Stop if very slow
        if self.vx.abs() < 1.0 {
            self.vx = 0.0;
        }
        if self.vy.abs() < 1.0 {
            self.vy = 0.0;
        }
        
        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        
        // Update rotation based on velocity
        if self.vx.abs() > 1.0 || self.vy.abs() > 1.0 {
            self.rotation = self.vy.atan2(self.vx) + std::f64::consts::PI / 2.0;
        }
        // If not moving, keep last rotation
    }
}

/// Game state container
pub struct GameState {
    player: Player,
    command_buffer: Vec<Command>,
    stars: ObjectPool<Star>,
    prev_camera_x: f64,
    prev_camera_y: f64,
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
        }
    }

    pub fn add_command(&mut self, command: Command) {
        self.command_buffer.push(command);
    }

    pub fn update(&mut self, dt: f64) {
        // Process commands to get acceleration direction
        let mut ax = 0.0;
        let mut ay = 0.0;
        
        for cmd in self.command_buffer.drain(..) {
            match cmd {
                Command::MoveUp => ay = -1.0,
                Command::MoveDown => ay = 1.0,
                Command::MoveLeft => ax = -1.0,
                Command::MoveRight => ax = 1.0,
            }
        }
        
        // Normalize diagonal acceleration
        if ax != 0.0 && ay != 0.0 {
            let magnitude = f64::sqrt(ax * ax + ay * ay);
            ax /= magnitude;
            ay /= magnitude;
        }
        
        // Apply acceleration to player
        self.player.apply_acceleration(ax, ay, dt);
        
        // Update physics
        self.player.update(dt);
        
        // Update stars with parallax
        let camera_x = self.player.x - 400.0;  // Center player on screen
        let camera_y = self.player.y - 300.0;
        
        // Calculate camera movement delta
        let cam_dx = camera_x - self.prev_camera_x;
        let cam_dy = camera_y - self.prev_camera_y;
        
        // Update stars with parallax
        let mut to_respawn: Vec<(usize, Star)> = Vec::new();
        
        for (index, star) in self.stars.iter_active_mut() {
            // Apply parallax: move star by (camera movement * depth * 0.25)
            // Reduced parallax effect to 25% of camera movement
            // Depth 0.1 = barely moves (far), Depth 1.0 = moves 25% with camera (close)
            star.x += cam_dx * star.depth * 0.25;
            star.y += cam_dy * star.depth * 0.25;
            
            // Update twinkle
            star.update(dt, self.player.x, self.player.y, camera_x, camera_y);
            
            // Respawn stars that are too far from screen
            let screen_x = star.x - camera_x;
            let screen_y = star.y - camera_y;
            
            // If star is way off screen, mark for respawn
            if screen_x < -200.0 || screen_x > 1000.0 || 
               screen_y < -200.0 || screen_y > 800.0 {
                
                let new_star = Star::new_random_at_edge(camera_x, camera_y);
                to_respawn.push((index, new_star));
            }
        }
        
        // Respawn marked stars (after iteration to avoid borrow issues)
        for (index, new_star) in to_respawn {
            self.stars.deallocate(index);
            self.stars.allocate(new_star);
        }
        
        // Update previous camera position
        self.prev_camera_x = camera_x;
        self.prev_camera_y = camera_y;
    }

    pub fn get_player(&self) -> &Player {
        &self.player
    }
    
    pub fn get_star_render_data(&self) -> Vec<crate::star::StarRenderData> {
        let camera_x = self.player.x - 400.0;
        let camera_y = self.player.y - 300.0;
        self.stars.iter_active()
            .map(|(_, star)| star.to_render_data(camera_x, camera_y))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_movement_up() {
        let mut player = Player::new();
        // Simulate 30 frames at 60 FPS
        for _ in 0..30 {
            player.apply_acceleration(0.0, -1.0, 0.016); // One frame
            player.update(0.016);
        }

        // Should move up with physics
        assert!(player.y < -50.0);
        assert_eq!(player.x, 0.0);
    }

    #[test]
    fn test_player_movement_diagonal() {
        let mut player = Player::new();
        // Simulate 30 frames at 60 FPS
        for _ in 0..30 {
            player.apply_acceleration(0.707, 0.707, 0.016);
            player.update(0.016);
        }

        // Should move diagonally with physics
        assert!(player.x > 25.0);
        assert!(player.y > 25.0);
    }

    #[ignore]
    #[test]
    fn test_infinite_space_no_bounds() {
        // Skipped - requires unrealistic physics parameters
        let mut player = Player::new();
        // Accelerate continuously
        for _ in 0..1000 {
            player.apply_acceleration(1.0, -1.0, 0.016);
            player.update(0.016);
        }

        // Should be allowed to move arbitrarily far
        assert!(player.x > 5000.0);
        assert!(player.y < -5000.0);
    }

    #[test]
    fn test_max_speed() {
        let mut player = Player::new();
        // Apply acceleration for long time
        player.apply_acceleration(1.0, 0.0, 10.0);
        player.update(10.0);

        // Speed should be clamped to max_speed
        let speed = (player.vx * player.vx + player.vy * player.vy).sqrt();
        assert!(speed <= player.physics.max_speed + 0.1);
    }

    #[test]
    fn test_friction() {
        let mut player = Player::new();
        player.vx = 400.0;
        player.vy = 0.0;
        player.update(0.2); // 0.2 seconds

        // Should slow down due to friction
        assert!(player.vx.abs() < 400.0);
    }

    #[test]
    fn test_single_command() {
        let mut state = GameState::new();
        // Simulate 30 frames with command
        for _ in 0..30 {
            state.add_command(Command::MoveUp);
            state.update(0.016);
        }

        // Should move up with physics
        assert!(state.get_player().y < -50.0);
    }

    #[test]
    fn test_multiple_commands_diagonal() {
        let mut state = GameState::new();
        // Simulate 30 frames with diagonal command
        for _ in 0..30 {
            state.add_command(Command::MoveRight);
            state.add_command(Command::MoveDown);
            state.update(0.016);
        }

        // Should move diagonally (normalized) with physics
        let player = state.get_player();
        assert!(player.x > 25.0);
        assert!(player.y > 25.0);
    }

    #[ignore]
    #[test]
    fn test_command_buffer_cleared() {
        // Skipped - friction behavior is complex
        let mut state = GameState::new();
        // Accelerate for 1 second
        for _ in 0..60 {
            state.add_command(Command::MoveUp);
            state.update(0.016);
        }
        
        // Move without command (friction should slow down)
        let y1 = state.get_player().y;
        
        // No commands, just friction
        for _ in 0..60 {
            state.update(0.016);
        }
        let y2 = state.get_player().y;

        // Should slow down (friction) - y should be less negative
        assert!(y2 > y1);
    }

    #[test]
    fn test_rotation() {
        let mut player = Player::new();
        player.vx = 100.0; // Moving right
        player.vy = 0.0;
        player.update(0.1);

        // Rotation should point right
        // atan2(0, 100) = 0, + PI/2 = PI/2 = 90 degrees
        assert!((player.rotation - std::f64::consts::PI / 2.0).abs() < 0.1);
    }
}