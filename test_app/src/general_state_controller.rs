use agent_mcap::Context;

struct GeneralStateController {

}

// TODO : Créer un State Controller qui va publier pour tout le monde un état général, et qui va 
// s'abonner à des événements d'un peu tout le monde pour changer cet état.
// On va avoir notre première dépendance circulaire, certainement que le topic "Evenement" sera créé dans le main, 
// et donc passé en référence ici... Ou pas, car c'est lui qui sera MPSC, donc créable dans le new ici, à passer aux 
// autres noeuds lors de leurs constructions.
impl GeneralStateController {
    pub fn new(name: &str, context: &mut Context) -> Self {
        GeneralStateController {  }
    }
}