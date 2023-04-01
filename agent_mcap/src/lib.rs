use std::{io::BufWriter, sync::Arc, borrow::Cow, collections::BTreeMap};
use anyhow::Result;
use log::error;
use mcap::{Writer, Schema, Channel, records::MessageHeader};

use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::Mutex;

pub struct Context {
    writer: Arc<Mutex<Writer<'static, BufWriter<std::fs::File>>>>,    
}

impl Context {
    pub fn new() -> Result<Self> {
        let out = Arc::new(Mutex::new(Writer::new(
            BufWriter::new(std::fs::File::create("out.mcap")?)
        )?));

        Ok(Context {
            writer: out
        })
    }

    pub async fn advertise<T>(&mut self, topic_name: &str) -> Result<TopicSPMC<T>> 
    where T: Sync + Send + Clone + Serialize + JsonSchema + 'static {
        // Ici on se permet de mettre Clone dans les contraintes du type pour pouvoir avoir des structures contenant des String.
        // Don on va clonner le message à chaque fois (ce qui est plus long qu'une copie !).
        let (tx, rx) = tokio::sync::watch::channel(Option::<T>::None);
        // si on doit sauvegarder le topic, l'enregistrer ici...
        let mut rx2 = rx.clone();
        let jsonschema = schemars::schema_for!(T);
        let the_schema = Cow::from(serde_json::to_string_pretty(&jsonschema)?.as_bytes().to_owned());
        let schema = Arc::new(Schema {
            name: "exail.PositionMsg".to_string(), // TODO mettre le nom du type "T" ici...
            encoding: "jsonschema".to_string(),
            data: the_schema
        });

        let channel = Channel {
            topic: topic_name.to_string(),
            schema: Some(schema),
            message_encoding: String::from("json"),
            metadata: BTreeMap::default()
        };
        let writer = self.writer.clone();
        let channel_id = writer.lock().await.add_channel(&channel)?; 
        tokio::spawn(async move {
            let mut sequence = 0_u32;
            while rx2.changed().await.is_ok() {
                let data = rx2.borrow().clone();
                if let Some(to_save) = data {
                    match serde_json::to_string(&to_save) {
                        Ok(data_serialized) => {
                           writer.lock().await.write_to_known_channel(&MessageHeader{
                                channel_id: channel_id,
                                sequence: sequence,
                                log_time: 0,
                                publish_time: 0
                            }, data_serialized.as_bytes());
                            sequence = sequence + 1;
                        },
                        Err(_) => {}
                    };
    
                }
            }
        });
        Ok(TopicSPMC { name: topic_name.to_string(), tx: tx, rx: rx })
    }       
}

impl Drop for Context {
    fn drop(&mut self) {
        let writer = self.writer.clone();
        tokio::task::spawn(async move {let _ = writer.lock().await.finish();});
    }
}

pub struct TopicSPMC<T> {
    pub name: String,
    pub tx: tokio::sync::watch::Sender<Option<T>>,
    pub rx: tokio::sync::watch::Receiver<Option<T>>    
}

impl<T> TopicSPMC<T> {    
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<Option<T>> {
        self.rx.clone()
    }
    pub fn publish(&self, value: T) {
        if let Err(_) = self.tx.send(Some(value)) {
            error!("Channel {} closed !", self.name);
        }
    }
}

/* 
TODO : Créer un MPSC pour les événements qui vont être émits par tout le monde à destination du State Controller.
pub struct TopicMPSC<T> {
    pub name: String,
    pub tx: tokio::sync::mpsc::Sender<Option<T>>,
    pub rx: tokio::sync::mpsc::Receiver<Option<T>>    
}

impl<T> TopicMPSC<T> {        
    pub fn publish(&self, value: T) {
        if let Err(_) = self.tx.send(Some(value)) {
            error!("Channel {} closed !", self.name);
        }
    }
}
*/

#[async_trait::async_trait]
pub trait INode 
where Self: Sized
{
    async fn new(name: &str, context: &mut Context) -> Result<Self>;
    async fn subscribe_to_data(&mut self);
    async fn task(&mut self);
}

pub fn synchronize_data<T>(published: &TopicSPMC<T>, destination: Arc<Mutex<Option<T>>>)
where T: Clone + Send + Sync + 'static {
    // Subscribing to data :
    let mut published_clone = published.subscribe();
    tokio::spawn(async move {
        while published_clone.changed().await.is_ok() {
            let v = published_clone.borrow().clone();
            *destination.lock().await = v;
        }
    });
}