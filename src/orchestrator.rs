use bevy::prelude::*;
use common_game::protocols::messages::*;
use crossbeam_channel::*;
use std::collections::HashMap;
use std::thread::JoinHandle;

#[derive(Resource)]
pub struct Orchestrator {
    orch_tx: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_rx: HashMap<u32, Receiver<PlanetToOrchestrator>>,
    planet_handle: HashMap<u32, JoinHandle<()>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            orch_tx: HashMap::new(),
            planet_rx: HashMap::new(),
            planet_handle: HashMap::new(),
        }
    }

    pub fn add_op_tx(&mut self, id: u32, tx: Sender<OrchestratorToPlanet>) {
        self.orch_tx.insert(id, tx);
    }
    pub fn add_po_rx(&mut self, id: u32, rx: Receiver<PlanetToOrchestrator>) {
        self.planet_rx.insert(id, rx);
    }
    pub fn add_planet_handle(&mut self, id: u32, handle: JoinHandle<()>) {
        self.planet_handle.insert(id, handle);
    }

    pub fn join_planet_id(&mut self, id: u32) {
        match self.planet_handle.remove(&id).unwrap().join() {
            Ok(()) => info!("planet {id} joined successfully"),
            Err(e) => error!("and error {:?} occurred while joining the planet {id}", e),
        }
    }

    pub fn send_to_planet_id(&self, id: u32, msg: OrchestratorToPlanet) {
        info!("attempting to send message {:?} to planet {id}", &msg);
        match self.orch_tx.get(&id).unwrap().send(msg) {
            Ok(()) => {
                info!("Sended message to planet {id}")
            }
            Err(e) => warn!(
                "an error {:?} occurred while sending message to planet {id}",
                e
            ),
        }
    }

    pub fn recv_from_planet_id(
        &self,
        id: u32,
    ) -> Result<PlanetToOrchestrator, crossbeam_channel::RecvTimeoutError> {
        self.planet_rx
            .get(&id)
            .unwrap()
            .recv_timeout(std::time::Duration::from_millis(100))
    }
}
