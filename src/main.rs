#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod hsv2rgb;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::random;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use hsv2rgb::hsv2rgb;

const COLOR_BITSHIFT: u32 = 12;
const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;
const RATE_MUL: u32 = 4;
const ANTS: u32 = 2;
const COLOR_INC: u16 = 1;
const COLOR_INC2: u32 = 512;
const SHOW_ANTS: bool = false;
const FLIP_MAX: u64 = 60000;
const HUE2_START: u32 = 1400 << COLOR_BITSHIFT;
const HUE2_END: u32 = 900 << COLOR_BITSHIFT;
const HUE2: bool = true;
const FADE_COUNT: u32 = 2000;

struct Ant {
    x: i32,
    y: i32,
    direction: u8,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    state: Vec<Vec<u32>>,
    hue: Vec<Vec<u16>>,
    hue2: Vec<Vec<u32>>,
    ants: Vec<Ant>,
    flip_count: u64,
    steps_since_draw: u32,
    reverse: bool,
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new((WIDTH * 4) as f64, (HEIGHT * 4) as f64);
        WindowBuilder::new()
            .with_title("ANTS!!!")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {:?}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            for _ in 0..RATE_MUL{
                world.update();
            }
            window.request_redraw();
        }
    });
}

impl Ant {
    fn new() -> Self {
        let mut ret = Self {
            x: 0,
            y: 0,
            direction: random::<u8>() %4,
        };
        ret.randomize();
        ret
    }

    fn update(&mut self, state: &mut Vec<Vec<u32>>, hue: &mut Vec<Vec<u16>>, hue2: &mut Vec<Vec<u32>>, steps_since_draw: u32, reverse: bool) {
        self.turn(state);
        self.flip(state);
        self.color(hue, hue2, steps_since_draw, reverse);
        self.advance();
    }

    fn reverse(&mut self){
        self.direction = (self.direction + 2) % 4;
        self.advance();
    }

    fn turn(&mut self, state: &mut Vec<Vec<u32>>){
        // Turn
        if state[self.x as usize][self.y as usize] == 0 {
            if self.direction == 3 {self.direction = 0;}
            else {self.direction += 1;}
        }
        else {
            if self.direction == 0 {self.direction = 3;}
            else {self.direction -= 1;}
        }
    }

    fn flip(&mut self, state: &mut Vec<Vec<u32>>){
        // Flip
        //state[self.x as usize][self.y as usize] ^= true;
		if state[self.x as usize][self.y as usize] == 0 {
			state[self.x as usize][self.y as usize] = FADE_COUNT;
		}
		else {
			state[self.x as usize][self.y as usize] = 0;
		}
    }

    fn color(&mut self, hue: &mut Vec<Vec<u16>>, hue2: &mut Vec<Vec<u32>>, steps_since_draw: u32, reverse: bool){
        // Color
        if reverse == false {
            hue[self.x as usize][self.y as usize] =  hue[self.x as usize][self.y as usize].wrapping_add(COLOR_INC);
            hue[self.x as usize][self.y as usize] %= 1536;
        } else {
            let mut hue_temp = hue[self.x as usize][self.y as usize] as i32;
            hue_temp -= COLOR_INC as i32;
            hue[self.x as usize][self.y as usize] = hue_temp.rem_euclid(1536) as u16;
        }
        hue2[self.x as usize][self.y as usize] = (HUE2_START + (COLOR_INC2 * steps_since_draw)) % (1536 << COLOR_BITSHIFT);
    }

    fn advance(&mut self){
        // Move
        match self.direction {
        0 => self.x += 1,
        1 => self.y -= 1,
        2 => self.x -= 1,
        3 => self.y += 1,
        _ => (),
        }
        if self.x < 0 {self.x = WIDTH as i32 - 1;}
        if self.y < 0 {self.y = HEIGHT as i32 - 1;}
        if self.x >= WIDTH as i32 {self.x = 0;}
        if self.y >= HEIGHT as i32 {self.y = 0;}
    }

    fn randomize(&mut self){
        self.x = (rand::random::<u32>() % (WIDTH as u32)) as i32;
        self.y = (rand::random::<u32>() % (HEIGHT as u32)) as i32;
        self.direction = rand::random::<u8>() % 4;
    }
}

impl World {
    fn new() -> Self {
        let mut retval = Self {
            state: vec![],
            hue: vec![],
            hue2: vec![],
            ants: vec![],
            steps_since_draw: RATE_MUL as u32,
            flip_count: 0,
            reverse: false,
        };
        for _ in 0..WIDTH{
            retval.state.push(vec![0; HEIGHT as usize]);
            retval.hue.push(vec![0; HEIGHT as usize]);
            retval.hue2.push(vec![HUE2_END; HEIGHT as usize]);
        }
        for _ in 0..ANTS{
            retval.ants.push(
                Ant::new()
            );
        }
        retval
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.steps_since_draw > 0 {self.steps_since_draw -= 1;}
        if self.reverse == false{
            for ant in self.ants.iter_mut(){
                ant.update(&mut self.state, &mut self.hue, &mut self.hue2, self.steps_since_draw, self.reverse);
            }
        } else {
            for ant in self.ants.iter_mut().rev(){
                ant.update(&mut self.state, &mut self.hue, &mut self.hue2, self.steps_since_draw, self.reverse);
            }
        }
		/*
        self.flip_count += 1;
        if self.flip_count >= FLIP_MAX{
            self.flip_count = 0;
            self.reverse ^= true;
            if self.reverse == false{
                for ant in self.ants.iter_mut(){
                    ant.reverse();
                    ant.randomize();
                }
            } else {
                for ant in self.ants.iter_mut().rev(){
                    ant.reverse();
                }
            }	
        }
		*/
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&mut self, frame: &mut [u8]) {
        let mut x: usize = 0;
        let mut y: usize = 0;
        for pixel in frame.chunks_exact_mut(4) {
            let rgba: [u8; 4] = if {
                SHOW_ANTS && self.ants.iter().any(|j| (j.x as usize == x && j.y as usize == y))
            } {
                [0xffu8, 0xffu8, 0xffu8, 0xffu8]
            } else {
                if HUE2 == true {
                    let (r, g, b) = hsv2rgb((self.hue2[x][y] >> COLOR_BITSHIFT) as u16, 255, 255);
                    [r, g, b, 0xffu8]
                } else {
                    let (r, g, b) = hsv2rgb(self.hue[x][y], 255, 255);
                    [r, g, b, 0xffu8]
                }
            };
			if self.state[x][y] != 0 {self.state[x][y] -= 1;}
            let hue2_old = self.hue2[x][y];
            self.hue2[x][y] += COLOR_INC2 * RATE_MUL;
            if hue2_old <= HUE2_END && self.hue2[x][y] > HUE2_END { self.hue2[x][y] = HUE2_END; } else { self.hue2[x][y] %= 1536 << COLOR_BITSHIFT; }
            x += 1;
            if x >= WIDTH as usize {
                x = 0;
                y += 1
            }
            if y >= HEIGHT as usize { y = 0; }

            pixel.copy_from_slice(&rgba);
        }
        self.steps_since_draw = RATE_MUL;
    }
}