#![warn(warnings)]

use hdk::prelude::*;
// use hdk::prelude::holo_hash::*;
use holochain::conductor::config::ConductorConfig;
use holochain::sweettest::{SweetConductorBatch, SweetDnaFile};
use holochain::test_utils::consistency_10s;

use mews_integrity::*;

const DNA_FILEPATH: &str = "../../workdir/clutter.dna";
const ZOME_NAME: &str = "clutter";

#[tokio::test(flavor = "multi_thread")]
async fn trustd_feed_is_based_on_follow_topics() {
    // Use prebuilt DNA file
    let dna_path = std::env::current_dir().unwrap().join(DNA_FILEPATH);
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(3, ConductorConfig::default()).await;
    let apps = conductors.setup_app("clutter", &[dna]).await.unwrap();
    conductors.exchange_peer_info().await;

    let ((ann,), (bob,), (cat,)) = apps.into_tuples();

    let ann_zome = ann.zome(ZOME_NAME);
    let bob_zome = bob.zome(ZOME_NAME);
    let cat_zome = cat.zome(ZOME_NAME);

    // let ann_conductor = conductors[0].clone();
    // let bob_conductor = conductors[1].clone();
    // let cat_conductor = conductors[2].clone();

    let _entry_hash: EntryHash = conductors[0]
        .call(
            &ann_zome,
            "follow",
            FollowInput {
                agent: bob.agent_pubkey().clone(),
                follow_topics: vec![FollowTopicInput {
                    topic: String::from("kittens"),
                    weight: String::from("1.0"),
                }],
            },
        )
        .await;

    consistency_10s([&ann, &bob, &cat]).await;

    // let applet = AppletInstance {
    //     custom_name: String::from("custom name"),
    //     description: String::from("description"),
    //     logo_src: None,
    //     devhub_happ_release_hash: fixt!(EntryHash),
    //     devhub_gui_release_hash: fixt!(EntryHash),

    //     network_seed: None,
    //     properties: BTreeMap::new(), // Segmented by RoleId
    //     dna_hashes: BTreeMap::new(), // Segmented by RoleId
    // };

    // let _entry_hash: EntryHash = conductors[0]
    //     .call(&ann_zome, "register_applet_instance", applet)
    //     .await;

    // let all_applets: Vec<Record> = conductors[1]
    //     .call(&bob_zome, "get_applets_instances", ())
    //     .await;

    // assert_eq!(all_applets.len(), 1);
}

// #![warn(warnings)]

// use futures::future;

// use hdk::prelude::*;
// use holochain::sweettest::{
//     SweetAgents, SweetAppBatch, SweetCell, SweetConductor, SweetConductorBatch, SweetDnaFile,
// };

// use mews_integrity::*;

// const DNA_FILEPATH: &str = "../../workdir/clutter.dna";

// // TESTS:

// #[tokio::test(flavor = "multi_thread")]
// pub async fn trustd_feed_is_based_on_follow_() {
//     let (conductor, _ann, cell1): (SweetConductor, AgentPubKey, SweetCell) =
//         setup_1_conductor().await;

//     let long_mew = std::iter::repeat('a').take(200).collect::<String>();
//     let create_mew_input = CreateMewInput {
//         text: Some(long_mew),
//         mew_type: MewType::Original,
//         links: None,
//     };

//     let mew_hash: ActionHash = conductor
//         .call(&cell1.zome("mews"), "create_mew", create_mew_input)
//         .await;

//     let bytes = mew_hash.get_raw_39();
//     let leading_bytes = bytes.get(..3).unwrap();
//     assert_eq!(leading_bytes, &[132, 41, 36]);
// }

// // SETUP:

// async fn setup_1_conductor() -> (SweetConductor, AgentPubKey, SweetCell) {
//     let dna = SweetDnaFile::from_bundle(std::path::Path::new(DNA_FILEPATH))
//         .await
//         .unwrap();

//     let mut conductor = SweetConductor::from_standard_config().await;

//     let holo_core_agent = SweetAgents::one(conductor.keystore()).await;
//     let app1 = conductor
//         .setup_app_for_agent("app", holo_core_agent.clone(), &[dna.clone()])
//         .await
//         .unwrap();

//     let cell1 = app1.into_cells()[0].clone();

//     let agent_hash = holo_core_agent.into_inner();
//     let agent = AgentPubKey::from_raw_39(agent_hash).unwrap();

//     (conductor, agent, cell1)
// }

// pub async fn setup_conductors(n: usize) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
//     let dna = SweetDnaFile::from_bundle(std::path::Path::new(DNA_FILEPATH))
//         .await
//         .unwrap();

//     let mut conductors = SweetConductorBatch::from_standard_config(n).await;

//     let all_agents1: Vec<holochain::core::AgentPubKey> =
//         future::join_all(conductors.iter().map(|c| SweetAgents::one(c.keystore()))).await;

//     let all_agents2: Vec<AgentPubKey> = all_agents1
//         .iter()
//         .map(|holo_core_agent| {
//             let agent_hash = holo_core_agent.clone().into_inner();
//             AgentPubKey::from_raw_39(agent_hash).unwrap()
//         })
//         .collect();

//     let apps = conductors
//         .setup_app_for_zipped_agents("app", &all_agents1, &[dna])
//         .await
//         .unwrap();

//     conductors.exchange_peer_info().await;
//     (conductors, all_agents2, apps)
// }
