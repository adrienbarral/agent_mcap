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

    pub async fn advertise<T>(&mut self, topic_name: &str) -> Result<Topic<T>> 
    where T: Sync + Send + Clone + Serialize + JsonSchema + 'static {
        // Ici on se permet de mettre Clone dans les contraintes du type pour pouvoir avoir des structures contenant des String.
        // Don on va clonner le message à chaque fois (ce qui est plus long qu'une copie !).
        let (tx, rx) = tokio::sync::broadcast::channel(10);
        // si on doit sauvegarder le topic, l'enregistrer ici...
        let mut rx2 = tx.subscribe();
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
            // TODO : mettre tout ça dans une fonction, gérer l'erreur possible du write.
            // et ne rien faire si _to_save est à None.
            let mut sequence = 0_u32;
            while let Ok(to_save) = rx2.recv().await {
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
        });
        Ok(Topic { name: topic_name.to_string(), tx: tx })
    }       
}

impl Drop for Context {
    fn drop(&mut self) {
        let writer = self.writer.clone();
        tokio::task::spawn(async move {let _ = writer.lock().await.finish();});
    }
}

pub struct Topic<T> {
    pub name: String,
    pub tx: tokio::sync::broadcast::Sender<Option<T>>,
}

impl<T> Topic<T> {    
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Option<T>> {
        self.tx.subscribe()
    }
    pub fn publish(&self, value: T) {
        if let Err(_) = self.tx.send(Some(value)) {
            error!("Channel {} closed !", self.name);
        }
    }
}

#[async_trait::async_trait]
pub trait INode 
where Self: Sized
{
    async fn new(name: &str, context: &mut Context) -> Result<Self>;
    async fn subscribe_to_data(&mut self);
    async fn task(&mut self);
}

pub fn synchronize_data<T>(published: &Topic<T>, destination: Arc<Mutex<Option<T>>>)
where T: Clone + Send + Sync + 'static {
    // Subscribing to data :
    let mut published_clone = published.subscribe();
    tokio::spawn(async move {
        while let Ok(v) = published_clone.recv().await {
            *destination.lock().await = v;
        }
    });
}