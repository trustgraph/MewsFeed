#![warn(warnings)]

use futures::future;

use hdk::prelude::*;
use holochain::sweettest::{
    SweetAgents, SweetAppBatch, SweetCell, SweetConductor, SweetConductorBatch, SweetDnaFile,
};

use mews_integrity::*;

const DNA_FILEPATH: &str = "../../workdir/clutter.dna";

// TESTS:

#[tokio::test(flavor = "multi_thread")]
pub async fn mew_can_be_200_chars() {
    let (conductor, _alice, cell1): (SweetConductor, AgentPubKey, SweetCell) =
        setup_1_conductor().await;

    let long_mew = std::iter::repeat('a').take(200).collect::<String>();
    let create_mew_input = CreateMewInput {
        text: Some(long_mew),
        mew_type: MewType::Original,
        links: None,
    };

    let mew_hash: ActionHash = conductor
        .call(&cell1.zome("mews"), "create_mew", create_mew_input)
        .await;

    let bytes = mew_hash.get_raw_39();
    let leading_bytes = bytes.get(..3).unwrap();
    assert_eq!(leading_bytes, &[132, 41, 36]);
}

async fn setup_1_conductor() -> (SweetConductor, AgentPubKey, SweetCell) {
    let dna = SweetDnaFile::from_bundle(std::path::Path::new(DNA_FILEPATH))
        .await
        .unwrap();

    let mut conductor = SweetConductor::from_standard_config().await;

    let holo_core_agent = SweetAgents::one(conductor.keystore()).await;
    let app1 = conductor
        .setup_app_for_agent("app", holo_core_agent.clone(), &[dna.clone()])
        .await
        .unwrap();

    let cell1 = app1.into_cells()[0].clone();

    let agent_hash = holo_core_agent.into_inner();
    let agent = AgentPubKey::from_raw_39(agent_hash).unwrap();

    (conductor, agent, cell1)
}

pub async fn setup_conductors(n: usize) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
    let dna = SweetDnaFile::from_bundle(std::path::Path::new(DNA_FILEPATH))
        .await
        .unwrap();

    let mut conductors = SweetConductorBatch::from_standard_config(n).await;

    let all_agents1: Vec<holochain::core::AgentPubKey> =
        future::join_all(conductors.iter().map(|c| SweetAgents::one(c.keystore()))).await;

    let all_agents2: Vec<AgentPubKey> = all_agents1
        .iter()
        .map(|holo_core_agent| {
            let agent_hash = holo_core_agent.clone().into_inner();
            AgentPubKey::from_raw_39(agent_hash).unwrap()
        })
        .collect();

    let apps = conductors
        .setup_app_for_zipped_agents("app", &all_agents1, &[dna])
        .await
        .unwrap();

    conductors.exchange_peer_info().await;
    (conductors, all_agents2, apps)
}
