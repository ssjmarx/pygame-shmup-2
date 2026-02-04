/// Movement commands from Python to Rust
#[derive(Debug, Clone, Copy)]
pub enum Command {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    // Future phases will add:
    // Shoot,
    // Boost,
    // Etc.
}