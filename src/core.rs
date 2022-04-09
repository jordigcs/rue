
use crate::actors::{ActorSystem, ActorID, ActorKey, ActorMessage};

    use std::{
        collections::{HashSet, VecDeque},
        mem::{self, Discriminant},
        sync::Mutex,
    };
    use vulkano::instance::{Instance, InstanceCreateInfo};
    use vulkano_win::VkSurfaceBuild;
    use winit::{
        event::{self, Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    };

    #[derive(Debug)]
    pub struct GameVersion(pub u8, pub u8, pub u8);
    #[derive(Debug)]
    pub struct GameInfo {
        pub name: String,
        pub version: GameVersion,
    }

    impl Default for GameInfo {
        fn default() -> Self {
            Self {
                name: String::from("My Rue Game"),
                version: GameVersion(0, 1, 0),
            }
        }
    }
    pub struct Game {
        pub running: bool,
        pub game_info: GameInfo,
        pub core_systems: [ActorSystem; 3],
        pub systems: Vec<ActorSystem>,
        pub event_loop: Option<EventLoop<()>>,
        pub window: Option<Window>,
    }

    pub struct GameState {
        pub running: bool,
    }

    lazy_static! {
        pub static ref GAME_STATE: Mutex<GameState> = Mutex::new(GameState { running: false });
    }

    pub fn create_game(game_info: GameInfo) -> Game {
        Game {
            running: false,
            game_info,
            core_systems: [ActorSystem::new(), ActorSystem::new(), ActorSystem::new()],
            systems: vec![ActorSystem::new()],
            window: None,
            event_loop: None,
        }
    }

    impl Game {
        pub fn main(&mut self) -> &mut ActorSystem {
            &mut self.core_systems[0]
        }

        pub fn run(mut self) {
            println!("Rue Game Initialized!\nGame Info: {:?}", self.game_info);
            GAME_STATE.lock().unwrap().running = true;
            let event_loop = EventLoop::new();

            // Vulkan initialization
            let required_extensions = vulkano_win::required_extensions();
            let instance = Instance::new(InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            })
            .expect("failed to create instance");

            let surface = WindowBuilder::new()
                .build_vk_surface(&event_loop, instance.clone())
                .unwrap();
            surface.window().set_title(&self.game_info.name);
            let min_delta: f32 = 1.0 / 5.0;
            let mut current_time = std::time::Instant::now();
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } => {
                        if window_id == surface.window().id() {
                            *control_flow = ControlFlow::Exit;
                        }
                        println!("{:?}", window_id);
                    }
                    _ => (),
                }

                self.core_systems.iter_mut().for_each(|sys| {
                    sys.propagate_messages();
                });

                let new_time = std::time::Instant::now();
                let mut frame_time = (new_time - current_time).as_secs_f32();
                current_time = new_time;

                while frame_time > 0.0 {
                    let delta = frame_time.min(min_delta);
                    self.main().push_message(ActorMessage::Process(delta));
                    frame_time -= delta;
                    //overall_time += delta;
                }
            });
        }
    }