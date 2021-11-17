/*
use std::char;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use futures::{Future, SinkExt, stream::FuturesUnordered};
use talk::{crypto::{Identity}, unicast::{Message, Receiver, Sender, test::UnicastSystem}};
use tokio::time::Timeout;
use talk::time::test::join;
*/
/*
#[tokio::test]
async fn communicate(){
    const PEERS: usize = 8;
    let UnicastSystem {
        keys,
        senders,
        receivers
    } = UnicastSystem::<Msg>::setup(PEERS).await.into();

    let peers = Peer::compose(keys.clone(), senders.clone(), receivers);
    let handles = peers.into_iter()
        .map(|Peer {key: _ , mut sender, mut receiver}| {
            tokio::spawn(async move {
                for i in 0..PEERS {
                    let (message_sender, message, _) = receiver.receive().await;
                    let _ = receiver.receive().await;
                    if message.1 {
                        //tokio::time::sleep(Duration::from_secs(1)).await;
                        println!("Peer {} got from peer {}: \n      {}", i, message.2, message.0);
                    }
                   //let sender = sender.clone();
                    //sender.send(message_sender, (String::from("Got it!"), true,  i)).await.unwrap();
                }
            })

        })
        .collect::<Vec<_>>();
        let targets = keys.clone();
        let senders = senders.clone();
        let _ = senders.iter()
            .map(|s| {
                let mut i = 0;
                targets.iter()
                    .map(move |target| {
                        let s = s.clone();
                        let target = target.clone();
                        i += 1;
                        let res = async move {
                            s.send(target, (format!("Hello, it's peer {} !", i), true, i)).await.unwrap();
                        };
                        res
                    })
            })
            .flatten()
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;


    join(handles).await.unwrap()

}


#[tokio::test]
async fn constant_all_to_all_strong() {
    const PEERS: usize = 8;

    let UnicastSystem {
        keys,
        senders,
        receivers,
    } = UnicastSystem::<Identity>::setup(PEERS).await.into();
    let mut correspondances: HashMap<Identity, usize> = HashMap::new();
    let targets = keys.clone();
    let mut i = 0;
    for target in targets {
        correspondances.insert(target, i);
        i +=1 ;
    }
    let handles = receivers
        .into_iter()
        .map(|mut receiver| {
            tokio::spawn(async move {
                for i in 0..PEERS {
                    let (_, message, acknowledger) =
                        receiver.receive().await;
                    let _ = receiver.receive().await;

                    println!("Receiver {} receives from sender {:?}", i, message);
                    acknowledger.strong();
                }
            })
        })
        .collect::<Vec<_>>();
        senders
            .iter()
            .map(|sender| {
                keys.iter().map(move |key| {
                    let sender = sender.clone();
                    let key = key.clone();
                    async move { sender.send(key, key).await.unwrap() }
                })
            })
            .flatten()
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

    join(handles).await.unwrap();
}

async fn readMap(map: Arc<HashMap<char, char>>, c: char) -> char {
    let res = map.get(&c);
    res.unwrap().clone()
}


#[tokio::test]
async fn hashMapConcurrent(){
    const shift: u32 = 12;
    let mut map: HashMap<char, char> = HashMap::<_, _>::new();
    for i in 0..26{
        map.insert(char::from_u32(i + 'a' as u32).unwrap(), char::from_u32((i + shift) % 26 + 'a' as u32).unwrap());
    };

    for (key, value) in map.iter(){
        println!("({}, {})", key, value);
    }

   let map = map;
   let map = Arc::new(map);
   for _ in 0..10 {
       tokio::spawn(readMap(map.clone(), 'c'));
   }
}*/
