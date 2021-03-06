use hecs::*;
use web::*;

struct Game {
    time: i32,
    ctx: CanvasContext,
    canvas_width: i32,
    canvas_height: i32,
    width: i32,
    height: i32,
    direction: Direction,
    world: World,
    head: Entity,
    made_move: bool,
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

// ECS components
struct SnakeHead(i32);
struct SnakeBody(i32);
struct Food;
struct Position(i32, i32);
struct Color(String);

const MAP_WIDTH: i32 = 30;
const MAP_HEIGHT: i32 = 30;
const ITERATION_TIME: i32 = 100;

impl Game {
    fn new() -> Game {
        // create graphics context
        let screen = get_element_by_id("screen");
        let width: f64 = get_property(&screen, "width");
        let height: f64 = get_property(&screen, "height");
        let ctx = CanvasContext::from_canvas_element(&screen);
        // create snake
        let mut world = World::new();
        let head = world.spawn((
            SnakeHead(1),
            Color(GREEN.to_string()),
            Position(MAP_WIDTH / 2, MAP_HEIGHT / 2),
        ));
        let mut g = Game {
            time: 0,
            ctx,
            canvas_width: width as i32,
            canvas_height: height as i32,
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            direction: Direction::Down,
            head,
            world,
            made_move: false,
        };
        g.reset();
        g
    }

    fn instance() -> MutexGuard<'static, Game> {
        lazy_static::lazy_static! {
            static ref SINGLETON: Mutex<Game> = {
                Mutex::new(Game::new())
            };
        }
        SINGLETON.lock()
    }

    fn start() {
        add_event_listener(DOM_BODY, "keydown", |event| {
            let key_down_event = KeyDownEvent::from_event(event);
            let key_code = key_down_event.key_code();
            Game::instance().key_down(key_code);
        });

        request_animation_loop(|delta| match Game::instance().run(delta) {
            Err(e) => console_error(&e.to_string()),
            _ => (),
        });
    }

    fn reset(&mut self) {
        self.ctx
            .clear_rect(0, 0, self.canvas_width, self.canvas_height);
        self.world.clear();
        self.head = self.world.spawn((
            SnakeHead(1),
            Color(GREEN.to_string()),
            Position(MAP_WIDTH / 2, MAP_HEIGHT / 2),
        ));
        self.spawn_food();
    }

    fn spawn_food(&mut self) {
        // create initial food
        self.world.spawn((
            Food,
            Color(RED.to_string()),
            Position(
                (random() * MAP_WIDTH as f64) as i32,
                (random() * MAP_HEIGHT as f64) as i32,
            ),
        ));
    }

    fn key_down(&mut self, key_code: u32) {
        if self.made_move {
            return;
        }
        self.made_move = true;
        match key_code {
            87 | 38 => {
                if let Direction::Down = self.direction {
                } else {
                    self.direction = Direction::Up
                }
            }
            68 | 39 => {
                if let Direction::Left = self.direction {
                } else {
                    self.direction = Direction::Right
                }
            }
            83 | 40 => {
                if let Direction::Up = self.direction {
                } else {
                    self.direction = Direction::Down
                }
            }
            65 | 37 => {
                if let Direction::Right = self.direction {
                } else {
                    self.direction = Direction::Left
                }
            }
            _ => (),
        };
    }

    fn run(&mut self, delta: f64) -> Result<(), ComponentError> {
        self.time += delta as i32;
        if self.time > ITERATION_TIME {
            self.time %= ITERATION_TIME;
            self.move_snake_system()?;
            self.eat_system()?;
        }
        self.render_system();
        self.made_move = false;
        Ok(())
    }

    fn move_snake_system(&mut self) -> Result<(), ComponentError> {
        let (last_head_pos, next_head_pos) = {
            let mut pos = self.world.get_mut::<Position>(self.head)?;
            let p = Position(pos.0, pos.1);
            match self.direction {
                Direction::Up => pos.1 -= 1,
                Direction::Right => pos.0 += 1,
                Direction::Down => pos.1 += 1,
                Direction::Left => pos.0 -= 1,
            }
            (p, Position(pos.0, pos.1))
        };
        let mut body_to_remove = vec![];
        let mut bit_tail = false;
        if next_head_pos.0 < 0
            || next_head_pos.1 < 0
            || next_head_pos.0 > self.width
            || next_head_pos.1 > self.height
        {
            self.reset();
            return Ok(());
        }
        for (id, (body, pos)) in &mut self.world.query::<(&mut SnakeBody, &Position)>() {
            body.0 -= 1;
            if body.0 == 0 {
                body_to_remove.push(id);
            } else {
                if pos.0 == next_head_pos.0 && pos.1 == next_head_pos.1 {
                    bit_tail = true;
                    break;
                }
            }
        }
        if bit_tail {
            self.reset();
            return Ok(());
        }
        for b in body_to_remove.into_iter() {
            self.world.despawn(b)?;
        }
        let snake_level = self.world.get::<SnakeHead>(self.head)?.0;
        self.world.spawn((
            SnakeBody(snake_level),
            Color(FORESTGREEN.to_string()),
            last_head_pos,
        ));
        Ok(())
    }

    fn render_system(&self) {
        self.ctx
            .clear_rect(0, 0, self.canvas_width, self.canvas_height);
        for (_id, (pos, color)) in &mut self.world.query::<(&Position, &Color)>() {
            self.ctx.set_fill_color(&color.0);
            self.ctx.fill_rect(
                pos.0 * (self.canvas_width / MAP_WIDTH),
                pos.1 * (self.canvas_height / MAP_HEIGHT),
                self.canvas_width / MAP_WIDTH,
                self.canvas_height / MAP_HEIGHT,
            );
        }
    }

    fn eat_system(&mut self) -> Result<(), ComponentError> {
        let (head_x, head_y) = {
            let p = self.world.get::<Position>(self.head)?;
            (p.0, p.1)
        };
        let mut food_to_remove = None;
        for (id, (_, pos)) in &mut self.world.query::<(&Food, &Position)>() {
            if pos.0 == head_x && pos.1 == head_y {
                food_to_remove = Some(id);
                break;
            }
        }
        if let Some(id) = food_to_remove {
            {
                self.world.despawn(id)?;
            }
            {
                let mut head = self.world.get_mut::<SnakeHead>(self.head)?;
                head.0 += 1;
            }
            self.spawn_food();
        }
        Ok(())
    }
}

#[no_mangle]
pub fn main() {
    Game::start();
}
