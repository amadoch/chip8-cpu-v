use std::env;
use std::fs::File;
use std::io::Read;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use cli_app_1::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGTH: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 30;

fn main() {
    // tomar el rom como argumento
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usa como argumento la ruta del juego: '/home/user_example/games/game.rom'");
        return;
    };
    // Bootstrap SDL2
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();
    // preparar ventana
    let window = video_subsystem
        .window("Chip 8 Emu", WINDOW_WIDTH, WINDOW_HEIGTH)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    // inyectar capa de renderizado (dibujo o canvas pues...)
    let mut canvas = window
        .into_canvas()
        .build()
        .unwrap();
    // limpiar y actualizar renderizado
    canvas.clear();
    canvas.present();
    // cargar el sistema de eventos
    let mut event_pump = sdl_ctx.event_pump().unwrap();
    // Bootstrap del chip 8
    let mut chip_8_core = Emu::new();
    let mut rom = File::open(&args[1]).expect("Imposible abrir el archivo...");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip_8_core.load(&buffer);
    // MainLoop del emulador
    'gameloop: loop {
        for ev in event_pump.poll_iter() {
            match ev {
                Event::Quit { .. } => {
                    break 'gameloop;
                },
                _ => ()
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            chip_8_core.tick();
        }
        chip_8_core.tick_timers();
        draw_screen(&mut chip_8_core, &mut canvas);
    }

}

fn draw_screen(emu: &mut Emu, canvas: &mut Canvas<Window>) {
    // Limpias todo en negro
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    // Sacar frame
    let screen_buff = emu.get_display();
    // Procesar frame
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buff.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new(
                (x * SCALE) as i32,
                (y * SCALE) as i32,
                SCALE,
                SCALE
            );
            canvas.fill_rect(rect).unwrap();
        }
    }
    // Poner frame en pantalla
    canvas.present();
}
