use std::{time::Duration, io::BufWriter, collections::BTreeMap, borrow::Cow, sync::Arc};

use anyhow::Result;
use mcap::{Channel, Writer, Schema, records:: MessageHeader};
use schemars::JsonSchema;
use serde::Serialize;

use tokio::{sync::{watch::{Sender, Receiver, channel}, Mutex}};

pub struct Context {
    writer: Arc<Mutex<Writer<'static, BufWriter<std::fs::File>>>>
}

impl Context {
    pub fn new() -> Result<Self> {
        let out = Arc::new(Mutex::new(Writer::new(
            BufWriter::new(std::fs::File::create("out.mcap")?)
        )?));

        Ok(Context {writer: out })
    }
    pub async fn advertise<T>(&mut self, topic_name: &str) -> Result<Topic<T>> 
    where T: Sync + Send + Serialize + Copy + JsonSchema + 'static {
        let (tx, mut rx) = channel(Option::<T>::None);
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
                let data = *rx2.borrow();
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
        Ok(Topic { tx: tx, rx: rx })
    }
    
}

impl Drop for Context {
    fn drop(&mut self) {
        let writer = self.writer.clone();
        tokio::task::spawn(async move {let _ = writer.lock().await.finish();});
    }
}

pub struct Topic<T> {
    pub tx: Sender<Option<T>>,
    pub rx: Receiver<Option<T>>    
}

impl<T> Topic<T> {
    pub fn subscribe(&self) -> Receiver<Option<T>> {
        self.rx.clone()
    }
    pub fn publish(&self, value: T) -> Result<(), tokio::sync::watch::error::SendError<Option<T>>> {
        self.tx.send(Some(value))
    }
}

#[async_trait::async_trait]
pub trait INode 
where Self: Sized
{
    async fn new(name: &str, context: &mut Context) -> Result<Self>;
    async fn task(&mut self);
}
