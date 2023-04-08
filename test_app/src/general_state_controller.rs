use std::sync::Arc;

use agent_mcap::{Context, Topic};
use anyhow::Result;
use crate::messages::{Event, StateMsg};


pub struct GeneralStateController {
    name: String,
    pub event: Topic<Event>,
    pub state: Topic<StateMsg>
}

// TODO : Créer un State Controller qui va publier pour tout le monde un état général, et qui va 
// s'abonner à des événements d'un peu tout le monde pour changer cet état.
// On va avoir notre première dépendance circulaire, certainement que le topic "Evenement" sera créé dans le main, 
// et donc passé en référence ici... Ou pas, car c'est lui qui sera MPSC, donc créable dans le new ici, à passer aux 
// autres noeuds lors de leurs constructions.
impl GeneralStateController {
    pub async fn new(name: &str, context: &mut Context) -> Result<Arc<Self>> {
        let event: Topic<Event> = context.advertise("event").await?;
        let mut rx = event.subscribe();
        let res = Arc::new(GeneralStateController {
            name: String::from(name), 
            event: event,
            state: context.advertise("state").await?
        });

        let state_controller = res.clone();
        // Ici on voudrait lier le lifetime de res à celui du spawn...
        tokio::spawn(async move {
            while let Ok(received) = rx.recv().await {
                if let Some(event) = received {
                    state_controller.on_event_received(&event); 
                }
            }
        });
        Ok(res)
    }

    fn on_event_received(&self, event: &Event) {
        match event.event {
            crate::messages::EventType::GPS_FIX_LOST | crate::messages::EventType::NO_GPS_DATA_RECEIVED => {
                self.state.publish(StateMsg { state: crate::messages::State::OUT_OF_ORDER(String::from("No GPS")) });
            },
            crate::messages::EventType::TOO_FAR_OF_TRAJECTORY => {
                self.state.publish(StateMsg { state: crate::messages::State::REACHING });
            }
        }
    }
}