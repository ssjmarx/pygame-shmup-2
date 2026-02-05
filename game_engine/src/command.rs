/// Movement commands from Python to Rust
#[derive(Debug, Clone, Copy)]
pub enum Command {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    
    // Movement mode commands
    ToggleAltMode(bool),         // Alt key state (no resistance)
    ToggleBoostMode(bool),       // Shift key state (boost acceleration)
    ToggleControlMode(bool),     // Ctrl key state (precision mode)
    
    // Mouse targeting
    SetMouseTarget(f64, f64, f64, f64),    // Mouse position + camera position (screen coords)
    
    // Shooting commands
    SetTargetEntity(Option<usize>),        // Set target entity ID for tracking shots
    StartShootingTracking,                  // Start shooting tracking bullets
    StopShootingTracking,                   // Stop shooting tracking bullets
    StartAutoFire,                          // Start autofiring
    StopAutoFire,                           // Stop autofiring
}
