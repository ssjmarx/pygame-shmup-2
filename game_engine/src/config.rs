/// Configuration for gun behavior
#[derive(Debug, Clone)]
pub struct GunConfig {
    /// Rotation speed in radians per second
    pub rotation_speed: f64,
    
    /// Maximum rotation limit in degrees (±half this amount from forward)
    pub rotation_limit_degrees: f64,
    
    /// Recoil decay rate (2.0 = 0.5s decay to zero)
    pub recoil_decay_rate: f64,
    
    /// Initial autofire cooldown in seconds
    pub autofire_cooldown_start: f64,
    
    /// Minimum autofire cooldown in seconds (at full spool)
    pub autofire_cooldown_min: f64,
    
    /// Rate of spool-up (seconds per second)
    pub autofire_spool_rate: f64,
    
    /// Rate of spool-down (seconds per second)
    pub autofire_spool_down_rate: f64,
    
    /// Movement compensation scale factor
    pub movement_compensation_scale: f64,
    
    /// Maximum random recoil offset in radians
    pub recoil_random_offset_max: f64,
    
    /// Maximum recoil stack multiplier
    pub recoil_stack_multiplier: f64,
    
    /// Recoil angle jitter multiplier
    pub recoil_angle_multiplier: f64,
}

/// Configuration for projectile behavior
#[derive(Debug, Clone)]
pub struct ProjectileConfig {
    // Tracking bullet properties
    pub tracking_speed: f64,
    pub tracking_size: f64,
    pub tracking_length: f64,
    pub tracking_weight: f64,
    pub tracking_lifetime: f64,
    pub tracking_scan_interval: f64,
    pub tracking_scan_radius: f64,
    pub tracking_color: (u8, u8, u8),
    
    // Autofire bullet properties
    pub autofire_speed: f64,
    pub autofire_size: f64,
    pub autofire_length: f64,
    pub autofire_weight: f64,
    pub autofire_lifetime: f64,
    pub autofire_color: (u8, u8, u8),
    
    // Projectile mechanics
    pub steering_strength: f64,       // Radians per second
    pub projectile_angle_offset: f64,  // Offset to prevent collision
    pub tracking_recoil_amount: f64,
    pub autofire_recoil_amount: f64,
}

/// Configuration for player gun positions and behavior
#[derive(Debug, Clone)]
pub struct PlayerGunConfig {
    /// Left gun offset from player center (x, y)
    pub left_gun_offset: (f64, f64),
    
    /// Right gun offset from player center (x, y)
    pub right_gun_offset: (f64, f64),
    
    /// Gun visual length (pixels)
    pub gun_length: f64,
    
    /// Movement compensation multiplier
    pub movement_compensation: f64,
    
    /// Cooldown between tracking shots (seconds)
    pub tracking_cooldown: f64,
    
    /// Front overlap angle (radians) where both guns fire
    pub front_overlap_angle: f64,
    
    /// Rear overlap angle (radians) where both guns fire
    pub rear_overlap_angle: f64,
}

impl Default for GunConfig {
    fn default() -> Self {
        Self {
            rotation_speed: 4.0,              // Equal to player rotation
            rotation_limit_degrees: 200.0,     // ±100 degrees from forward
            recoil_decay_rate: 2.0,           // 0.5s decay
            autofire_cooldown_start: 0.5,      // 1 shot per 0.5s
            autofire_cooldown_min: 0.1,        // 1 shot per 0.1s
            autofire_spool_rate: 0.2,          // 2.0s to 0.1s (doubled)
            autofire_spool_down_rate: 0.2,      // 2.0s from 0.1s (doubled)
            movement_compensation_scale: 0.1,
            recoil_random_offset_max: 0.5,       // ±0.5 radians max
            recoil_stack_multiplier: 5.0,
            recoil_angle_multiplier: 0.2,
        }
    }
}

impl Default for ProjectileConfig {
    fn default() -> Self {
        Self {
            // Tracking bullet
            tracking_speed: 800.0,
            tracking_size: 6.0,
            tracking_length: 12.0,
            tracking_weight: 2.0,
            tracking_lifetime: 5.0,
            tracking_scan_interval: 0.1,        // 10 scans per second
            tracking_scan_radius: 200.0,
            tracking_color: (100, 150, 255),     // Bright blue
            
            // Autofire bullet
            autofire_speed: 1000.0,             // Faster than tracking
            autofire_size: 3.0,
            autofire_length: 6.0,
            autofire_weight: 0.5,
            autofire_lifetime: 2.0,
            autofire_color: (255, 200, 50),      // Yellow
            
            // Mechanics
            steering_strength: 10.0,            // Radians per second
            projectile_angle_offset: 0.05,       // ~3 degrees
            tracking_recoil_amount: 0.05,
            autofire_recoil_amount: 0.03,
        }
    }
}

impl Default for PlayerGunConfig {
    fn default() -> Self {
        Self {
            left_gun_offset: (7.5, 10.0),
            right_gun_offset: (-7.5, 10.0),
            gun_length: 10.0,                // Half of 20px height
            movement_compensation: 1.0,
            tracking_cooldown: 0.5,
            front_overlap_angle: std::f64::consts::PI / 4.0,  // 45° front
            rear_overlap_angle: std::f64::consts::PI / 4.0,   // 45° rear
        }
    }
}
