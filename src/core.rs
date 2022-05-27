use creek::{*, actors::{ActorTypes, ActorHandle, Actor, ActorID}};
use std::{
    collections::{HashSet, VecDeque},
    mem::{self, Discriminant},
    sync::Mutex, cell::RefMut,
};
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::{self, PhysicalDevice}, Device, DeviceCreateInfo, QueueCreateInfo, Features, DeviceExtensions}};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer::Renderer;

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

pub enum SceneEvent {

}

#[derive(Clone)]
pub struct SceneManager<T: ActorTypes + Clone> {
    pub cur_scene: Creek<T>,
    id: Option<ActorID>,
    actions: Vec<CreekAction>
}

impl<T: ActorTypes + Clone> Actor for SceneManager<T> {
    type Event = SceneEvent;

    fn receive_event(&mut self, event:Self::Event) {
        todo!()
    }

    fn get_creek_actions(&self) -> &Vec<CreekAction> {
        &self.actions
    }

    fn get_id(&self) -> Option<actors::ActorID> {
        self.id
    }
}

#[derive(Clone)]
pub enum CoreSystems<T: ActorTypes + Clone> {
    Renderer,
    SceneManager(SceneManager<T>)
}

impl<T: ActorTypes + Clone> creek::actors::ActorTypes for CoreSystems<T> {
    fn propogate_global_event(&mut self, event:&GlobalEvent) -> Option<&Vec<CreekAction>> {
        todo!()
    }
}

pub struct Game<T: ActorTypes + Clone + 'static> {
    pub running: bool,
    pub game_info: GameInfo,
    pub core_system: Creek<CoreSystems<T>>,
    pub scene_manager: ActorHandle<CoreSystems<T>>,
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Window>,
}

pub struct GameState {
    pub running: bool,
}

lazy_static! {
    pub static ref GAME_STATE: Mutex<GameState> = Mutex::new(GameState { running: false });
}

pub fn create_game<T: ActorTypes + Clone>(game_info: GameInfo) -> Game<T> {
    let mut g = Game {
        running: false,
        game_info,
        core_system: Creek::new(),
        scene_manager: ActorHandle::default(),
        window: None,
        event_loop: None,
    };
    g.scene_manager = g.core_system.add_actor(CoreSystems::SceneManager(SceneManager {
        cur_scene: Creek::<T>::new(),
        id: None,
        actions: Vec::new(),
    }));
    g
}

impl<T: ActorTypes + Clone + 'static > Game<T> {

    pub fn scene(&mut self) -> RefMut<Option<CoreSystems<T>>> {
        self.scene_manager.borrow_actor_mut()
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

        let renderer = Renderer::new(surface.clone());

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
                },
                Event::MainEventsCleared => { // Render
                    renderer.render();
                }
                _ => (),
            }

            self.scene_manager.edit_actor(|sm| {
                if let CoreSystems::SceneManager(s) = sm {
                    s.cur_scene.propagate_events();
                }
            });

            let new_time = std::time::Instant::now();
            let mut frame_time = (new_time - current_time).as_secs_f32();
            current_time = new_time;

            while frame_time > 0.0 {
                let delta = frame_time.min(min_delta);
                self.scene_manager.edit_actor(|s_manager| {
                    if let CoreSystems::SceneManager(s) = s_manager {
                        s.cur_scene.push_event(creek::GlobalEventType::Update(delta), None);
                    }
                });
                frame_time -= delta;
                //overall_time += delta;
            }
        });
    }
}
