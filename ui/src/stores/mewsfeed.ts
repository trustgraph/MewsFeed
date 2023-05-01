import {
  callZome,
  getFeedMewAndContext,
  mewsFeed,
  basketFeed,
  MewsFn,
} from "@/services/mewsfeed-dna";
import { Mew, FeedMew } from "@/types/types";
import { isSameHash } from "@/utils/hash";
import { showError } from "@/utils/notification";
import { ActionHash } from "@holochain/client";
import { defineStore } from "pinia";

export const MEWSFEED_ROLE_NAME = "mewsfeed";
export const MEWS_ZOME_NAME = "mews";

export const makeUseMewsfeedStore = () => {
  return defineStore("mewsfeed", {
    state: () => ({
      mewsFeed: [] as FeedMew[],
      basketFeed: [] as TrustFeedMew[],
      isLoadingMewsFeed: false,
      isLoadingBasketFeed: false,
    }),
    actions: {
      async fetchMewsFeed() {
        try {
          this.isLoadingMewsFeed = true;
          this.mewsFeed = await mewsFeed();
        } catch (error) {
          showError(error);
        } finally {
          this.isLoadingMewsFeed = false;
        }
      },
      async fetchBasketFeed() {
        try {
          this.isLoadingBasketFeed = true;
          this.basketFeed = await basketFeed({
            now: new Date().getTime() * 1000, // microseconds since epoch
            oldestMewSeconds: null,
          });
          console.log({ basketFeed: this.basketFeed });
        } catch (error) {
          showError(error);
        } finally {
          this.isLoadingBasketFeed = false;
        }
      },
      async reloadMew(actionHash: ActionHash) {
        try {
          const index = this.mewsFeed.findIndex((mew) =>
            isSameHash(actionHash, mew.action_hash)
          );
          if (index !== -1) {
            this.mewsFeed[index] = await getFeedMewAndContext(actionHash);
          }
        } catch (error) {
          showError(error);
        }
      },
      async createMew(mew: Mew) {
        return callZome(MewsFn.CreateMew, mew);
      },
    },
  });
};
