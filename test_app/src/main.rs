use std::{time::Duration, io::BufWriter, collections::BTreeMap, borrow::Cow, sync::Arc};

use anyhow::Result;
use mcap::{Channel, Writer, Schema, records:: MessageHeader};
use schemars::JsonSchema;
use serde::Serialize;

use tokio::{sync::{watch::{Sender, Receiver, channel}, Mutex}, time::sleep};

struct Context {
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

#[derive(Serialize, Clone, Copy, JsonSchema)]
struct PositionMsg{
    latitude: f64,
    longitude: f64,
    heading: f32,

}
pub struct Topic<T> {
    tx: Sender<Option<T>>,
    rx: Receiver<Option<T>>    
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
trait INode 
where Self: Sized
{
    async fn new(name: &str, context: &mut Context) -> Result<Self>;
    async fn task(&mut self);
}
struct GPSNode {
    name: String,
    gps_topic: Topic<PositionMsg>
}

#[async_trait::async_trait]
impl INode for GPSNode {
    async fn new(name: &str, context: &mut Context) -> Result<Self> {
        Ok(GPSNode { name: String::from(name),
        gps_topic: context.advertise("gps_pos").await?
        })
    }
    async fn task(&mut self){
        loop{
            sleep(Duration::from_millis(100)).await;
            let _ = self.gps_topic.tx.send(Some(
                PositionMsg{latitude: 43.000, longitude: 6.000, heading: 10.0}
            ));
        }
    }
    // TODO : Implémenter drop pour interrompre la tâche quand on détruit le Node...
    // Peut être pas un trait du coup : https://stackoverflow.com/questions/71541765/rust-async-drop/71741467#71741467
}

#[tokio::main]
async fn main() -> Result<()>{
    let mut context = Context::new()?;
    let mut gps_node = GPSNode::new("gps_node", &mut context).await?;

    tokio::select! {
        _ = gps_node.task() => {},
        _ = tokio::signal::ctrl_c() => {} 
    };

    Ok(())
}
