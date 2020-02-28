use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameResult};

const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);

const COMMON_DIVISORS: [usize; 11] = [2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 200];

type Cells = [[CellState; 150]; 200];

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    ALIVE,
    DEAD,
}

enum Struct {
    BLINKER,
    GLIDER,
    GLIDER_GUN,
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("gol", "Last Game of Life")
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .expect("Could not create ggez context");

    let mut game = Gol::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut game) {
        Ok(_) => println!("Exit"),
        Err(e) => println!("Error : {}", e),
    }
}

struct Gol {
    auto_step: bool,
    display_grid: bool,
    zoom: usize,
    cells: Cells,
    building_cells: Option<CellStruct>,
}

struct CellStruct {
    cells: Cells,
    pos_x: usize,
    pos_y: usize,
}

impl CellStruct {
    pub fn new(s: Struct, x: usize, y: usize) -> CellStruct {
        let cells = &mut [[CellState::DEAD; 150]; 200];
        match s {
            Struct::BLINKER => {
                cells[0][0] = CellState::ALIVE;
                cells[1][0] = CellState::ALIVE;
                cells[2][0] = CellState::ALIVE;
            }
            Struct::GLIDER => {
                cells[2][0] = CellState::ALIVE;
                cells[2][1] = CellState::ALIVE;
                cells[1][1] = CellState::ALIVE;
                cells[1][2] = CellState::ALIVE;
                cells[0][0] = CellState::ALIVE;
            }
            Struct::GLIDER_GUN => {
                // Cube
                cells[0][4] = CellState::ALIVE;
                cells[0][5] = CellState::ALIVE;
                cells[1][4] = CellState::ALIVE;
                cells[1][5] = CellState::ALIVE;

                // Up branch
                cells[11][3] = CellState::ALIVE;
                cells[12][2] = CellState::ALIVE;
                cells[13][2] = CellState::ALIVE;

                // Facade 1
                cells[10][4] = CellState::ALIVE;
                cells[10][5] = CellState::ALIVE;
                cells[10][6] = CellState::ALIVE;

                // Back middle
                cells[14][5] = CellState::ALIVE;
                cells[15][3] = CellState::ALIVE;
                cells[16][4] = CellState::ALIVE;
                cells[16][5] = CellState::ALIVE;
                cells[17][5] = CellState::ALIVE;
                cells[16][6] = CellState::ALIVE;
                cells[15][7] = CellState::ALIVE;

                // Down branch
                cells[11][7] = CellState::ALIVE;
                cells[12][8] = CellState::ALIVE;
                cells[13][8] = CellState::ALIVE;

                cells[20][2] = CellState::ALIVE;
                cells[20][3] = CellState::ALIVE;
                cells[20][4] = CellState::ALIVE;
                cells[21][2] = CellState::ALIVE;
                cells[21][3] = CellState::ALIVE;
                cells[21][4] = CellState::ALIVE;
                cells[22][1] = CellState::ALIVE;
                cells[24][1] = CellState::ALIVE;
                cells[24][0] = CellState::ALIVE;
                cells[22][5] = CellState::ALIVE;
                cells[24][5] = CellState::ALIVE;
                cells[24][6] = CellState::ALIVE;

                cells[34][2] = CellState::ALIVE;
                cells[34][3] = CellState::ALIVE;
                cells[35][2] = CellState::ALIVE;
                cells[35][3] = CellState::ALIVE;
            }
        }

        CellStruct {
            cells: *cells,
            pos_x: x,
            pos_y: y,
        }
    }

    fn update_pos(&mut self, x: usize, y: usize) {
        self.pos_x = x;
        self.pos_y = y;
    }
}

impl Gol {
    pub fn new(_ctx: &mut Context) -> Gol {
        let cells = &mut [[CellState::DEAD; 150]; 200];

        Gol {
            auto_step: false,
            display_grid: true,
            zoom: 1,
            cells: *cells,
            building_cells: None,
        }
    }

    fn draw_grid(&mut self, ctx: &mut Context) -> GameResult<()> {
        let color = [0.3, 0.3, 0.3, 1.0].into();
        for p in (0..SCREEN_SIZE.0 as i32).step_by(self.grid_width() / 2) {
            let l = graphics::Mesh::new_line(
                ctx,
                &[
                    na::Point2::new(0.0, p as f32),
                    na::Point2::new(SCREEN_SIZE.0, p as f32),
                ],
                1.0,
                color,
            )?;
            let c = graphics::Mesh::new_line(
                ctx,
                &[
                    na::Point2::new(p as f32, 0.0),
                    na::Point2::new(p as f32, SCREEN_SIZE.1),
                ],
                1.0,
                color,
            )?;
            graphics::draw(ctx, &l, (na::Point2::new(0.0, p as f32),))?;
            graphics::draw(ctx, &c, (na::Point2::new(p as f32, 0.0),))?;
        }

        Ok(())
    }

    fn draw_cells(&mut self, ctx: &mut Context) -> GameResult<()> {
        for c in 0..self.cells.len() {
            for l in 0..self.cells[c].len() {
                let width = self.grid_width() as f32;
                if self.cells[c][l] == CellState::ALIVE {
                    let rect = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(0.0, 0.0, width + 1.0, width + 1.0),
                        [0.3, 0.1, 0.6, 1.0].into(),
                    )?;

                    graphics::draw(
                        ctx,
                        &rect,
                        (na::Point2::new(
                            c as f32 * width - 1.0,
                            l as f32 * width - 1.0,
                        ),),
                    )?;
                }
                if let Some(building_cells) = &self.building_cells {
                    if building_cells.cells[c][l] == CellState::ALIVE {
                        let rect = graphics::Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            graphics::Rect::new(0.0, 0.0, width + 1.0, width + 1.0),
                            [0.3, 0.3, 0.3, 1.0].into(),
                        )?;
                        graphics::draw(
                            ctx,
                            &rect,
                            (na::Point2::new(
                                (c as f32 + building_cells.pos_x as f32) * width - 1.0,
                                (l as f32 + building_cells.pos_y as f32) * width - 1.0,
                            ),),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn next_step(&mut self) {
        let mut next_cells = self.cells.clone();
        for c in 0..self.cells.len() {
            for l in 0..self.cells[c].len() {
                let cell = self.cells[c][l];

                match cell {
                    CellState::ALIVE | CellState::DEAD => {
                        let mut neighboors = [CellState::DEAD; 8];
                        if c > 0 && l > 0 && c < self.cells.len() - 2 && l < self.cells[c].len() - 2
                        {
                            neighboors[0] = self.cells[c - 1][l - 1];
                            neighboors[1] = self.cells[c][l - 1];
                            neighboors[2] = self.cells[c + 1][l - 1];
                            neighboors[3] = self.cells[c - 1][l];
                            neighboors[4] = self.cells[c + 1][l];
                            neighboors[5] = self.cells[c - 1][l + 1];
                            neighboors[6] = self.cells[c][l + 1];
                            neighboors[7] = self.cells[c + 1][l + 1];
                        }

                        let mut nb_alive_n = 0;
                        for n in 0..neighboors.len() {
                            if neighboors[n] == CellState::ALIVE {
                                nb_alive_n += 1;
                            }
                        }

                        match cell {
                            CellState::DEAD => {
                                if nb_alive_n == 3 {
                                    next_cells[c][l] = CellState::ALIVE;
                                }
                            }
                            CellState::ALIVE => {
                                if nb_alive_n < 2 || nb_alive_n > 3 {
                                    next_cells[c][l] = CellState::DEAD;
                                } else {
                                    next_cells[c][l] = CellState::ALIVE;
                                }
                            }
                        }
                    }
                }
            }
        }
        self.cells = next_cells;
    }

    fn start_building(&mut self, s: Struct) {
        let (mut x, mut y) = (0, 0);
        if let Some(building_cells) = &self.building_cells {
            x = building_cells.pos_x;
            y = building_cells.pos_y;
        }
        self.building_cells = Some(CellStruct::new(s, x, y));
    }
    fn build(&mut self) {
        if let Some(building_cells) = &self.building_cells {
            for c in 0..building_cells.cells.len() {
                for l in 0..building_cells.cells[c].len() {
                    let cell = building_cells.cells[c][l];
                    if cell == CellState::ALIVE {
                        let (x, y) = (c + building_cells.pos_x, l + building_cells.pos_y);
                        if x < self.cells.len() && y < self.cells[c].len() {
                            self.cells[x][y] = building_cells.cells[c][l];
                        }
                    }
                }
            }
            self.building_cells = None;
        }
    }

    fn zoom_in(&mut self) {
        if self.zoom < COMMON_DIVISORS.len() - 1 {
            self.zoom += 1;
        }
    }
    fn zoom_off(&mut self) {
        if self.zoom > 1 {
            self.zoom -= 1;
        }
    }
    fn toggle_auto_step(&mut self) {
        self.auto_step = !self.auto_step;
    }
    fn toggle_display_grid(&mut self) {
        self.display_grid = !self.display_grid;
    }

    fn grid_width(&mut self) -> usize {
        COMMON_DIVISORS[self.zoom]
    }
}

impl EventHandler for Gol {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.auto_step {
            self.next_step();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        if self.display_grid {
            self.draw_grid(ctx)?;
        }
        self.draw_cells(ctx)?;
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::PageUp => self.zoom_in(),
            KeyCode::PageDown => self.zoom_off(),
            KeyCode::A => self.toggle_auto_step(),
            KeyCode::G => self.toggle_display_grid(),
            KeyCode::Key1 => self.start_building(Struct::BLINKER),
            KeyCode::Key2 => self.start_building(Struct::GLIDER),
            KeyCode::Key3 => self.start_building(Struct::GLIDER_GUN),
            _ => (),
        }

        if let Some(cell_struct) = &mut self.building_cells {
            match keycode {
                KeyCode::Up => cell_struct.update_pos(cell_struct.pos_x, cell_struct.pos_y - 1),
                KeyCode::Down => cell_struct.update_pos(cell_struct.pos_x, cell_struct.pos_y + 1),
                KeyCode::Right => cell_struct.update_pos(cell_struct.pos_x + 1, cell_struct.pos_y),
                KeyCode::Left => cell_struct.update_pos(cell_struct.pos_x - 1, cell_struct.pos_y),
                KeyCode::Space => self.build(),
                _ => (),
            }
        } else {
            match keycode {
                KeyCode::Space => {
                    if !self.auto_step {
                        self.next_step();
                    }
                }
                _ => (),
            }
        }
    }
}
