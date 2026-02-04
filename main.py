import pygame
import sys
import math
from game_engine import GameEngine

# Constants
BASE_WIDTH = 800
BASE_HEIGHT = 600
FPS = 60
PLAYER_COLOR = (0, 255, 0)  # Green
PLAYER_SIZE = 20  # Reduced from 32
PLAYER_WIDTH = 15.0  # Updated per user request
PLAYER_HEIGHT = 20.0  # Updated per user request
BACKGROUND_COLOR = (20, 20, 30)  # Dark blue-gray

# Star colors (cool palette)
STAR_COLORS = {
    'white': (255, 255, 255),
    'light_blue': (173, 216, 230),
    'cyan': (0, 255, 255),
    'light_purple': (221, 160, 221),
    'pink': (255, 182, 193),
    'pale_yellow': (238, 232, 170)
}


def get_max_window_size():
    """Calculate maximum window size that fits on monitor while maintaining 4:3 aspect ratio"""
    info = pygame.display.Info()
    monitor_width = info.current_w
    monitor_height = info.current_h
    
    # 4:3 aspect ratio
    aspect_ratio = 4 / 3
    
    # Try to maximize within monitor bounds
    if monitor_width / monitor_height > aspect_ratio:
        # Monitor is wider than 4:3
        window_height = monitor_height * 0.9  # 90% of monitor height
        window_width = window_height * aspect_ratio
    else:
        # Monitor is taller than 4:3
        window_width = monitor_width * 0.9  # 90% of monitor width
        window_height = window_width / aspect_ratio
    
    return int(window_width), int(window_height)


class InputHandler:
    """Handle keyboard input and send commands to Rust engine"""

    def __init__(self, game_engine):
        self.game_engine = game_engine

    def handle(self, renderer):
        """Process all pygame events and send commands"""
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                return False

            # Handle window resize
            if event.type == pygame.VIDEORESIZE:
                renderer.handle_resize(event)

            # Handle key presses
            if event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    return False

        # Continuous input (held keys)
        keys = pygame.key.get_pressed()

        if keys[pygame.K_w] or keys[pygame.K_UP]:
            self.game_engine.send_command("move_up")
        if keys[pygame.K_s] or keys[pygame.K_DOWN]:
            self.game_engine.send_command("move_down")
        if keys[pygame.K_a] or keys[pygame.K_LEFT]:
            self.game_engine.send_command("move_left")
        if keys[pygame.K_d] or keys[pygame.K_RIGHT]:
            self.game_engine.send_command("move_right")

        return True


class Renderer:
    """Render game state"""

    def __init__(self, screen):
        self.screen = screen
        self.base_width = BASE_WIDTH
        self.base_height = BASE_HEIGHT
        self.scale_factor = screen.get_width() / self.base_width
        self.font = pygame.font.Font(None, int(36 * self.scale_factor))
    
    def handle_resize(self, event):
        """Handle window resize events while maintaining 4:3 aspect ratio"""
        if event.type == pygame.VIDEORESIZE:
            new_width, new_height = event.w, event.h
            
            # Maintain 4:3 aspect ratio
            actual_ratio = new_width / new_height
            target_ratio = 4 / 3
            
            if actual_ratio > target_ratio:
                # Too wide, adjust height
                new_height = new_width / target_ratio
            elif actual_ratio < target_ratio:
                # Too tall, adjust width
                new_width = new_height * target_ratio
            
            # Update screen
            self.screen = pygame.display.set_mode(
                (int(new_width), int(new_height)),
                pygame.RESIZABLE
            )
            self.scale_factor = new_width / self.base_width
            self.font = pygame.font.Font(None, int(36 * self.scale_factor))

    def draw_star(self, star, scale, cam_x_scaled, cam_y_scaled):
        """Draw a single star with scaling"""
        # Calculate screen position
        screen_x = int((star['x'] * scale) - cam_x_scaled)
        screen_y = int((star['y'] * scale) - cam_y_scaled)
        size = star['size'] * scale
        
        # Get base color
        base_color = STAR_COLORS[star['color']]
        
        # Apply twinkle (brightness modulation)
        twinkle_factor = 0.5 + (star['twinkle'] * 0.5)  # 0.5 to 1.0
        color = (
            int(base_color[0] * twinkle_factor),
            int(base_color[1] * twinkle_factor),
            int(base_color[2] * twinkle_factor)
        )
        
        # Draw based on shape
        shape = star['shape']
        
        if shape == 'circle':
            pygame.draw.circle(self.screen, color, (screen_x, screen_y), int(size))
        
        elif shape == 'four_point':
            # Draw 4-point star ✦
            points = []
            for i in range(4):
                angle = (i * math.pi / 2) - math.pi / 4  # Rotate 45 degrees
                outer_x = screen_x + math.cos(angle) * size
                outer_y = screen_y + math.sin(angle) * size
                inner_angle = angle + math.pi / 4
                inner_x = screen_x + math.cos(inner_angle) * size * 0.4
                inner_y = screen_y + math.sin(inner_angle) * size * 0.4
                points.extend([(outer_x, outer_y), (inner_x, inner_y)])
            
            pygame.draw.polygon(self.screen, color, points)
        
        elif shape == 'six_point':
            # Draw 6-point star ✶
            points = []
            for i in range(6):
                angle = (i * math.pi / 3) - math.pi / 6  # Rotate 30 degrees
                outer_x = screen_x + math.cos(angle) * size
                outer_y = screen_y + math.sin(angle) * size
                inner_angle = angle + math.pi / 6
                inner_x = screen_x + math.cos(inner_angle) * size * 0.4
                inner_y = screen_y + math.sin(inner_angle) * size * 0.4
                points.extend([(outer_x, outer_y), (inner_x, inner_y)])
            
            pygame.draw.polygon(self.screen, color, points)

    def render(self, render_data):
        """Render all game objects with scaling"""
        # Clear screen
        self.screen.fill(BACKGROUND_COLOR)

        # Extract render data
        player_x = render_data["player_x"]
        player_y = render_data["player_y"]
        cam_x = render_data["camera_x"]
        cam_y = render_data["camera_y"]
        player_rotation = render_data.get("player_rotation", -math.pi / 2)  # Default: pointing up
        stars = render_data.get("stars", [])  # Get star data if available

        # Apply scaling
        scale = self.scale_factor
        player_x_scaled = player_x * scale
        player_y_scaled = player_y * scale
        cam_x_scaled = cam_x * scale
        cam_y_scaled = cam_y * scale
        
        # Draw stars first (background layer)
        for star in stars:
            self.draw_star(star, scale, cam_x_scaled, cam_y_scaled)

        # Convert to screen coordinates
        screen_x = int(player_x_scaled - cam_x_scaled)
        screen_y = int(player_y_scaled - cam_y_scaled)

        # Draw player as a triangle (vector graphics style)
        # Use scaled size
        size = PLAYER_SIZE * scale
        width = (PLAYER_WIDTH / 2) * scale  # Half width
        height = (PLAYER_HEIGHT / 2) * scale  # Half height

        # Define base triangle (pointing up)
        # Vertices: tip, right corner, left corner
        base_vertices = [
            (0, -height),           # Tip (top)
            (width, height),         # Bottom right
            (-width, height),        # Bottom left
        ]

        # Rotate vertices
        rotated_vertices = []
        for vx, vy in base_vertices:
            # Rotation matrix
            rx = vx * math.cos(player_rotation) - vy * math.sin(player_rotation)
            ry = vx * math.sin(player_rotation) + vy * math.cos(player_rotation)
            rotated_vertices.append((screen_x + rx, screen_y + ry))

        # Draw filled triangle (black fill)
        pygame.draw.polygon(self.screen, (0, 0, 0), rotated_vertices)

        # Draw outline (green stroke, scaled width - thinner stroke)
        stroke_width = int(1.5 * scale)
        pygame.draw.polygon(self.screen, PLAYER_COLOR, rotated_vertices, stroke_width)

        # Draw debug info
        self.draw_debug_info(player_x, player_y, cam_x, cam_y)

        # Update display
        pygame.display.flip()

    def draw_debug_info(self, player_x, player_y, cam_x, cam_y):
        """Draw position and camera info on screen"""
        info_lines = [
            f"Player: ({player_x:.1f}, {player_y:.1f})",
            f"Camera: ({cam_x:.1f}, {cam_y:.1f})",
            "WASD/Arrows: Move",
            "ESC: Quit",
        ]

        for i, line in enumerate(info_lines):
            text = self.font.render(line, True, (255, 255, 255))
            self.screen.blit(text, (10, 10 + i * 30))


def main():
    """Main game loop"""
    # Initialize pygame
    pygame.init()
    
    # Auto-size window to monitor
    window_width, window_height = get_max_window_size()
    screen = pygame.display.set_mode((window_width, window_height), pygame.RESIZABLE)
    pygame.display.set_caption("Phase 1: Player Movement")
    clock = pygame.time.Clock()

    # Initialize Rust game engine
    game_engine = GameEngine()

    # Initialize systems
    input_handler = InputHandler(game_engine)
    renderer = Renderer(screen)

    # Main loop
    running = True
    while running:
        # Calculate delta time (in seconds)
        dt = clock.tick(FPS) / 1000.0

        # Handle input (includes resize events)
        running = input_handler.handle(renderer)

        # Update game state in Rust
        game_engine.update(dt)

        # Get render data from Rust
        render_data = game_engine.get_render_data()

        # Render
        renderer.render(render_data)

    # Cleanup
    pygame.quit()
    sys.exit()


if __name__ == "__main__":
    main()