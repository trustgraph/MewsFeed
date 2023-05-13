import { assert, test } from "vitest";
import { runScenario, pause } from "@holochain/tryorama";
import { Record } from "@holochain/client";
import { createMew } from "./common";
import { FeedMew, Mew, MewTypeName, FollowInput } from "../../../../ui/src/types/types";
import { mewsfeedAppBundleSource } from "../../utils";

test("Weighted followed creators mews should be ordered by topic weights descending order", async () => {
  await runScenario(
    async (scenario) => {
      // Set up the app to be installed
      const appSource = { appBundleSource: mewsfeedAppBundleSource };

      // Add players with the test app to the Scenario. The returned players can be destructured.
      const [ann, bob, cat] = (await scenario.addPlayersWithApps([
        appSource,
        appSource,
        appSource,
      ])).map((player) => {
        return {
          follow: (payload: FollowInput) => player.cells[0].callZome({
            zome_name: "mews",
            fn_name: "follow",
            payload
          })
        }
      });

      // Shortcut peer discovery through gossip and register all agents in every conductor of the scenario.
      await scenario.shareAllAgents();

      ann.follow( {
          agent: bob.pubkey.clone(),
          follow_topics: vec![FollowTopicInput {
              topic: String::from("holochain"),
              weight: String::from("1.0"),
          }],
          follow_other: false,
      })
      .await;

      ann.follow(FollowInput {
          agent: cat.pubkey.clone(),
          follow_topics: vec![FollowTopicInput {
              topic: String::from("holochain"),
              weight: String::from("0.5"),
          }],
          follow_other: false,
      })
      .await;

      ann.follow(FollowInput {
          agent: bob.pubkey.clone(),
          follow_topics: vec![FollowTopicInput {
              topic: String::from("blockchain"),
              weight: String::from("0.25"),
          }],
          follow_other: false,
      })
      .await;

      ann.follow(FollowInput {
          agent: cat.pubkey.clone(),
          follow_topics: vec![FollowTopicInput {
              topic: String::from("blockchain"),
              weight: String::from("0"),
          }],
          follow_other: false,
      })
      .await;

      bob.create_mew(CreateMewInput {
          mew_type: MewType::Original,
          text: Some(String::from("#holochain from bob, weight 1.0")),
          links: None,
      })
      .await;

      bob.create_mew(CreateMewInput {
          mew_type: MewType::Original,
          text: Some(String::from("#blockchain from bob, weight 0.25")),
          links: None,
      })
      .await;

      cat.create_mew(CreateMewInput {
          mew_type: MewType::Original,
          text: Some(String::from("#blockchain from cat, weight 0.0")),
          links: None,
      })
      .await;

      cat.create_mew(CreateMewInput {
          mew_type: MewType::Original,
          text: Some(String::from("#holochain from cat, weight 0.5")),
          links: None,
      })
      .await;

      consistency_10s([&(ann.cell.clone()), &(bob.cell.clone()), &(cat.cell.clone())]).await;

      let recommended_feed = ann
          .recommended(RecommendedInput {
              now: Timestamp::now(),
              oldest_mew_seconds: Some(60 * 60), // last hour
          })
          .await;

      assert_eq!(recommended_feed.len(), 4);
      assert_eq!(recommended_feed[0].feed_mew.mew.content.as_ref().unwrap().text, String::from("#holochain from bob, weight 1.0"));
      assert_eq!(recommended_feed[1].feed_mew.mew.content.as_ref().unwrap().text, String::from("#holochain from cat, weight 0.5"));
      assert_eq!(recommended_feed[2].feed_mew.mew.content.as_ref().unwrap().text, String::from("#blockchain from bob, weight 0.25"));
      assert_eq!(recommended_feed[3].feed_mew.mew.content.as_ref().unwrap().text, String::from("#blockchain from cat, weight 0.0"));


    },
    true,
    { timeout: 100000 }
  );
});
