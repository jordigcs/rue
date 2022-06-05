use cgmath::{Quaternion, Vector2};
use creek::{*, actors::{ActorTypes, ActorHandle, Actor, ActorID}};
use std::{
    sync::{Arc}, cell::{RefMut, RefCell}, rc::Rc,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}, dpi::{LogicalSize, PhysicalSize},
};

use crate::renderer::{Renderer, RenderConfig, RenderableInstance};


#[derive(Debug)]
pub struct WindowConfig {
    pub resizable:bool,
    pub maximizable: bool,
    window_size: (u32, u32),
}

impl WindowConfig {
    pub fn build(window_size: (u32, u32)) -> Self {
        WindowConfig {
            resizable: true,
            maximizable: true,
            window_size,
        }
    }

    pub fn fixed_size(mut self) -> Self {
        self.resizable = false;
        self.maximizable = false;
        self
    }
}

#[derive(Debug)]
pub struct GameVersion(pub u8, pub u8, pub u8);
#[derive(Debug)]
pub struct GameInfo {
    pub name: String,
    pub target_fps: u8,
    pub version: GameVersion,
    pub window_config: WindowConfig,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            name: String::from("My Rue Game"),
            target_fps: 60,
            version: GameVersion(0, 1, 0),
            window_config: WindowConfig::build((640, 480))
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
    pub room_manager: ActorHandle<CoreSystems<T>>,
    pub event_loop: Option<EventLoop<()>>,
    pub renderer: Option<Rc<RefCell<Renderer>>>,
    pub render_config: Option<Rc<RefCell<RenderConfig>>>,
    pub delta_time: f32,
    pub total_delta_time: f32,
}

pub struct GameState {
    pub running: bool,
}

impl<T: ActorTypes + Clone + 'static > Game<T> {
    pub fn new(game_info: GameInfo) -> Game<T> {
        let mut g = Game {
            running: false,
            game_info,
            core_system: Creek::new(),
            room_manager: ActorHandle::default(),
            event_loop: None,
            renderer: None,
            render_config: None,
            delta_time: 0.01,
            total_delta_time: 0.0,
        };
        g.room_manager = g.core_system.add_actor(CoreSystems::SceneManager(SceneManager {
            cur_scene: Creek::<T>::new(),
            id: None,
            actions: Vec::new(),
        }));
        g
    }

    pub fn with_render_config(mut self, render_config:RenderConfig) -> Self {
        self.render_config = Some(Rc::new(RefCell::new(render_config)));
        self
    }

    pub fn scene(&mut self) -> RefMut<Option<CoreSystems<T>>> {
        self.room_manager.borrow_actor_mut()
    }

    pub fn run(mut self) {
        println!("Rue Game Initialized!\nGame Info: {:?}", self.game_info);

        env_logger::init();
        let event_loop = EventLoop::new();
        let window = Rc::new(WindowBuilder::new()
            .build(&event_loop)
            .unwrap());
        window.set_title(&self.game_info.name);
        let phys_size = PhysicalSize::new(self.game_info.window_config.window_size.0, self.game_info.window_config.window_size.1);
        window.set_inner_size(phys_size);
        window.set_resizable(self.game_info.window_config.resizable);
        self.event_loop = Some(event_loop);


        let mut renderer = Rc::new(RefCell::new(pollster::block_on(Renderer::new(window, self.render_config))));
        self.renderer = Some(renderer.clone());

        let min_delta: f32 = 1.0 / (self.game_info.target_fps as f32);
        println!("{}. {}", self.game_info.target_fps, min_delta);
        let mut current_time = std::time::Instant::now();
        let mut accumulated_frame_time:f32 = 0.0;
        let mut dir:f64 = 1.0;
        match self.event_loop {
            Some(event_loop) => {
                event_loop.run(move |event, _, control_flow| {
                    *control_flow = ControlFlow::Poll;
                    match event {
                        Event::WindowEvent {
                            event,
                            window_id,
                        } => {
                            if window_id == renderer.borrow().window.id() {
                                match event {
                                    WindowEvent::CloseRequested => {
                                        *control_flow = ControlFlow::Exit;
                                    },
                                    WindowEvent::Resized(phys_size) => {
                                        renderer.borrow_mut().resize(phys_size);
                                    },
                                    WindowEvent::ScaleFactorChanged { new_inner_size: phys_size, .. } => {
                                        renderer.borrow_mut().resize(*phys_size);
                                    },
                                    _ => {}
                                }
                            }
                        }
                        Event::MainEventsCleared => {
        
                            let new_time = std::time::Instant::now();
                            let frame_time = (new_time - current_time).as_secs_f32();
                            current_time = new_time;
                            
                            accumulated_frame_time += frame_time;
        
                            while accumulated_frame_time > self.delta_time {
                                self.room_manager.edit_actor(|s_manager| {
                                    if let CoreSystems::SceneManager(s) = s_manager {
                                        s.cur_scene.push_event(creek::GlobalEventType::Update(self.delta_time), None);
                                        s.cur_scene.propagate_events();
                                    }
                                });
                                accumulated_frame_time -= self.delta_time;
                                self.total_delta_time += self.delta_time;
                                //overall_time += delta;
                                if renderer.borrow().render_config.borrow().clear_color.r > 1.0 {
                                    dir = -1.0;
                                }
                                else if renderer.borrow().render_config.borrow().clear_color.r < 0.0 {
                                    dir = 1.0;
                                }
                                renderer.borrow_mut().render_config.borrow_mut().clear_color.r += (self.delta_time as f64) * dir;
                            }
                            
                            match renderer.borrow_mut().render(vec![
                                RenderableInstance { position: Vector2::new(0.0, 0.0), rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0) }
                            ]) {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => renderer.borrow_mut().resize(renderer.borrow().window.inner_size()),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => (),
                    }
                });
            }
            None => {
                    log::error!("An error occured when trying to run the game. Event loop not initialized correctly.");
            }
        }
    }
}
