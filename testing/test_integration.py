from game_engine import GameEngine

def test_basic_movement():
    """Test that player can move in all directions"""
    engine = GameEngine()
    
    # Initial position
    data = engine.get_render_data()
    initial_x = data['player_x']
    initial_y = data['player_y']
    
    # Move up
    engine.send_command('move_up')
    engine.update(0.1)
    data = engine.get_render_data()
    assert data['player_y'] < initial_y, "Player should move up"
    
    # Move right
    engine.send_command('move_right')
    engine.update(0.1)
    data = engine.get_render_data()
    assert data['player_x'] > initial_x, "Player should move right"
    
    print("✓ Basic movement test passed")

def test_camera_following():
    """Test that camera follows player"""
    engine = GameEngine()
    
    # Get initial camera position
    initial_data = engine.get_render_data()
    initial_cam_x = initial_data['camera_x']
    initial_cam_y = initial_data['camera_y']
    
    # Move player
    for _ in range(50):
        engine.send_command('move_right')
        engine.send_command('move_down')
        engine.update(0.1)
    
    data = engine.get_render_data()
    
    # Camera should have moved
    assert data['camera_x'] > initial_cam_x, "Camera should move right with player"
    assert data['camera_y'] > initial_cam_y, "Camera should move down with player"
    
    # Player should have moved significantly
    assert data['player_x'] > 1000, "Player should have moved far right"
    assert data['player_y'] > 1000, "Player should have moved far down"
    
    print("✓ Camera following test passed")

def test_infinite_space():
    """Test that player can move arbitrarily far"""
    engine = GameEngine()
    
    # Move for a long time
    for _ in range(100):
        engine.send_command('move_right')
        engine.send_command('move_up')
        engine.update(0.1)
    
    data = engine.get_render_data()
    
    # Should be far from origin
    assert abs(data['player_x']) > 1000, "Player should move far from origin"
    assert abs(data['player_y']) > 1000, "Player should move far from origin"
    
    # Should still work
    engine.send_command('move_left')
    engine.update(0.1)
    new_data = engine.get_render_data()
    assert new_data['player_x'] < data['player_x'], "Movement should continue to work"
    
    print("✓ Infinite space test passed")

def test_diagonal_normalization():
    """Test that diagonal movement is normalized"""
    engine = GameEngine()
    
    # Move diagonally for 1 second
    for _ in range(60):  # ~1 second at 60 FPS
        engine.send_command('move_right')
        engine.send_command('move_up')
        engine.update(1.0 / 60.0)
    
    data = engine.get_render_data()
    
    # Calculate distance from origin
    distance = (data['player_x']**2 + data['player_y']**2)**0.5
    
    # Speed is 400 pixels/second, so after 1 second we should be ~400 pixels away
    # (accounting for some variation due to frame timing)
    assert 350 < distance < 450, f"Diagonal speed should be normalized (distance: {distance})"
    
    print("✓ Diagonal normalization test passed")

if __name__ == "__main__":
    print("Running integration tests...")
    print()
    
    test_basic_movement()
    test_camera_following()
    test_infinite_space()
    test_diagonal_normalization()
    
    print()
    print("All integration tests passed! ✓")