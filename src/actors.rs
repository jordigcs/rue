use crate::core::*;
    pub struct MessagesToRecieve(pub HashSet<Discriminant<ActorMessage>>);

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ActorKey {
        index:usize,
        generation:usize
    }
    
    impl MessagesToRecieve {
        pub fn new(messages: Vec<ActorMessage>) -> Self {
            let mut hashset = HashSet::<Discriminant<ActorMessage>>::new();
            for m in messages {
                hashset.insert(mem::discriminant(&m));
            }
            MessagesToRecieve(hashset)
        }
    }

    pub struct ActorID {
        pub key: ActorKey,
        messages_to_recieve: MessagesToRecieve,
        actor: Option<Box<dyn Actor>>,
        local_message_queue: VecDeque<ActorMessage>,
    }

    impl ActorID {
        pub fn get(&self) -> Option<&Box<dyn Actor>> {
            self.actor.as_ref()
        }
    }

        use std::{collections::{VecDeque, HashSet}, mem::{self, Discriminant}, sync::Mutex};

        pub const EMPTY_ACTOR_KEY : ActorKey = ActorKey {
            index: usize::MAX,
            generation: usize::MAX
        };

        #[derive(Debug, Clone, Copy)]
        pub enum ActorMessage {
            None,
            ActorDestroyed(ActorKey),
            ActorAddedToSystem(ActorKey),
            Created,
            Destroyed,
            Changed,
            Process(f32),
        }

        pub trait Actor : Send + Sync + 'static {
            fn set_actor_key(&mut self, actor_key:ActorKey);
            fn get_actor_key(&self) -> Option<ActorKey>;
            fn recieve_message(&mut self, system:&mut ActorSystem, message:&ActorMessage) -> ActorMessage;
            fn get_messages_to_recieve(&self) -> MessagesToRecieve;
        }

        pub struct ActorSystem {
            actors:Vec<ActorID>,
            global_message_queue:VecDeque<ActorMessage>
        }

        impl ActorSystem {
            pub fn new() -> Self {
                ActorSystem { actors: vec![], global_message_queue: VecDeque::new() }
            }

            pub fn add_actor(&mut self, actor:Box<dyn Actor>) -> ActorKey {
                let mut index:usize = self.actors.len();
                let mut generation:usize = 0;
                for (ind, actor) in self.actors.iter().enumerate() {
                    match actor.get() {
                        Some(..) => {},
                        None => {
                            index = ind;
                            generation += 1;
                        }
                    }
                }
                let actor_key = ActorKey { index, generation };
                let mut id = ActorID { key: actor_key, messages_to_recieve: actor.get_messages_to_recieve(), actor: Some(actor), local_message_queue: VecDeque::new() };
                id.local_message_queue.push_back(ActorMessage::Created);
                self.push_message(ActorMessage::ActorAddedToSystem(id.key));
                self.actors.push(id);
                self.propagate_messages();
                actor_key
            }

            pub fn push_message(&mut self, message:ActorMessage) {
                self.global_message_queue.push_back(message);
            }

            pub fn propgate_message(&mut self, message:ActorMessage) {
                self.push_message(message);
                self.propagate_messages();
            }

            fn _send_message_to_actor(actor_id:&mut ActorID, message:&ActorMessage, system:&mut ActorSystem) {
                if actor_id.messages_to_recieve.0.contains(&mem::discriminant(message)) {
                    if GAME_STATE.lock().expect("TEST").running {
                        //assert!(GAME_STATUS.lock().unwrap().running != true, "Game is not running but Rue is sending messages to actors. This should not be happening.");
                        match actor_id.actor.as_mut() {
                            Some(actor) => {
                                actor.recieve_message(system, message);
                            },
                            None => {}
                        }
                    } else {
                        actor_id.local_message_queue.push_back(*message);
                    }
                }
            }

            pub fn propagate_messages(&mut self) {
                if GAME_STATE.lock().expect("TEST").running {
                    while !self.global_message_queue.is_empty() {
                        let message = self.global_message_queue[0].clone();
                        for actor_id in self.actors.iter_mut() {
                            match actor_id.get() {
                                Some(..) => {
                                    ActorSystem::_send_message_to_actor(actor_id, &message, self);
                                },
                                None => {}
                            }
                        }
                        self.global_message_queue.pop_front();
                    }
                    // Local messages
                    for i in 0..self.actors.len() { 
                        while !self.actors[i].local_message_queue.is_empty() {
                            let msg = self.actors[i].local_message_queue[0].to_owned();
                            ActorSystem::_send_message_to_actor(&mut self.actors[i], &msg, self);
                            match msg {
                                ActorMessage::Destroyed => {
                                    self.actors.get_mut(i).unwrap().actor = None;
                                },
                                _ => {}
                            }
                            self.actors[i].local_message_queue.pop_front();
                        }
                    }
                }
            }

            pub fn get_actor(&self, actor_key:&ActorKey) -> Option<&Box<dyn Actor>> {
                let a_id = self.actors.get(actor_key.index);
                match a_id {
                    Some(v) => {
                        if v.key.generation == actor_key.generation {
                            v.actor.as_ref()
                        }
                        else {
                            None
                        }
                    },
                    None => {
                        None
                    }
                }
            }

            pub fn destroy_actor(&mut self, actor_key:&ActorKey) {
                let a_id = self.actors.get_mut(actor_key.index);
                match a_id {
                    Some(v) => {
                        if v.key.generation == actor_key.generation {                   
                            match &v.actor {
                                Some(..) => {
                                    ActorSystem::_send_message_to_actor(v, &ActorMessage::Destroyed,self);
                                },
                                None => {}
                            }
                        }
                        else {}
                    },
                    None => {}
                }
            }
        }

        impl Default for ActorSystem {
            fn default() -> Self {
                Self::new()
            }
        }
