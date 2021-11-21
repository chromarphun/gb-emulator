use sdl2::render::Canvas;
use sdl2::event::EventPump;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;

const color_map: [Color; 4] = [
    Color::RGB(15, 56, 15),
    Color::RGB(48, 98, 48),
    Color::RBG(139, 172, 15),
    Color::RBG(155, 188, 15)
];

const TILES_PER_ROW: usize = 16;
const TILES_PER_COLUMN: usize = 16;
const BG_MAP_SIZE: usize = 256;
const BG_TILE_WIDTH: usize = 8;
const BG_TILE_HEIGHT: usize = 8;
const BYTES_PER_TILE: usize = 16;
const BYTES_PER_TILE_ROW: usize = 2;
const SCREEN_PX_WIDTH: usize = 144;
const SCREEN_PX_HEIGHT: usize = 160;


struct PictureProcessingUnit {
    lcdc: Arc<Mutex<u8>>,
    stat: Arc<Mutex<u8>>,
    vram: Arc<Mutex<[u8; 6144]>>,
    tilemap_1: Arc<Mutex<[u8; 1024]>>,
    tilemap_2: Arc<Mutex<[u8; 1024]>>,
    screen: [u8; 23040],
    canvas: Canvas,
    event_pump: EventPump,
    scy: Arc<Mutex<u8>>,
    scx: Arc<Mutex<u8>>,
    ly: Arc<Mutex<u8>>,
    lyc: Arc<Mutex<u8>>,
    wy: Arc<Mutex<u8>>,
    wx: Arc<Mutex<u8>>,
    bgp: Arc<Mutex<u8>>, 
}

impl PictureProcessingUnit {
    fn new(lcdc: Arc<Mutex<u8>>,
        stat: Arc<Mutex<u8>>,
        vram: Arc<Mutex<[u8; 6144]>>,
        tilemap_1: Arc<Mutex<[u8; 1024]>>,
        tilemap_2: Arc<Mutex<[u8; 1024]>>,
        scy: Arc<Mutex<u8>>,
        scx: Arc<Mutex<u8>>,
        ly: Arc<Mutex<u8>>,
        lyc: Arc<Mutex<u8>>,
        wy: Arc<Mutex<u8>>,
        wx: Arc<Mutex<u8>>,
        bgp: Arc<Mutex<u8>>) -> PictureProcessingUnit {
            let screen = [0; 23040];
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem
                .window("Chip 18 Emulator", LONG_SIDE, SHORT_SIDE)
                .position_centered()
                .build()
                .unwrap();
        
            let mut canvas = window.into_canvas().build().unwrap();
            let mut event_pump = sdl_context.event_pump().unwrap();
            PictureProcessingUnit {
                lcdc,
                stat,
                vram,
                tilemap_1,
                tilemap_2,
                screen,
                canvas,
                event_pump: EventPump,
                scy,
                scx,
                ly,
                lyc,
                wy,
                wx,
                bgp, 
            }
        }
    fn get_tile_map_flag(&self) -> u8 {
        (self.lcdc.lock.unwrap() >> 3) & 1
    }
    fn get_tile_data_flag(&self) -> u8 {
        (self.lcdc.lock.unwrap() >> 4) & 1
    }
    fn run(&mut self) {
        'frame_loop: loop {
            {
                let vram = self.vram.lock().unwrap();

                for row in 0..160 {
                    let (scx, scy) = {
                        (self.scx.lock().unwrap(), self.scy.lock().unwrap())
                    };
                    let total_row = (scy + row) % BG_MAP_SIZE;
                    let mut column = 0;

                    let tilemap = if self.get_tile_map_flag() == 0 {
                        self.tilemap_1.lock().unwrap()
                    } else {
                        self.tilemap_2.lock().unwrap()
                    };
                    while column < SCREEN_PX_WIDTH {
                        let total_column = (scx + column) % BG_MAP_SIZE;
                        let tile_in_column = total_column % TILE_HEIGHT;
                        if self.get_tile_data_flag() == 0 {
                            let index = TILES_PER_ROW * BYTES_PER_TILE * (total_row / TILE_HEIGHT) 
                                + BYTES_PER_TILE * (total_column / TILE_WIDTH) 
                                + BYTES_PER_TILE_ROW * (total_row % TILE_HEIGHT);
                            let least_sig_byte = tilemap[index];
                            let most_sig_byte = tilemap[index + 1];
                            while tile_in_column < TILE_WIDTH && column < SCREEN_PX_WIDTH {
                                let color_index = (((most_sig_byte >> (TILE_WIDTH - tile_in_column - 1)) & 1) << 1) + ((most_sig_byte >> (TILE_WIDTH - tile_in_column - 1)) & 1);
                                canvas.set_draw_color(color_map[color_index]);
                                canvas
                                .fill_rect(Rect::new(column, row, 1, 1))
                                .expect("Failure to draw");
                            }
                        }
                    }

                }


            }

        }


    }
}
