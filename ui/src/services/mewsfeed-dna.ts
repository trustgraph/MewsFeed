import { useClientStore } from "@/stores";
import { MEWSFEED_ROLE_NAME, MEWS_ZOME_NAME } from "@/stores/mewsfeed";
import {
  ActionHash,
  AgentPubKey,
  AgentPubKeyB64,
  CallZomeRequest,
  decodeHashFromBase64,
} from "@holochain/client";
import {
  CreateMewInput,
  FeedMew,
  FeedOptions,
  Mew,
  RecommendedInput,
  TrustFeedMew,
} from "../types/types";

export enum MewsFn {
  CreateMew = "create_mew",
  GetMew = "get_mew",
  MewsFeed = "mews_feed",
  BasketFeed = "recommended",
  MewsBy = "mews_by",
  Follow = "follow",
  Followers = "followers",
  Following = "following",
  MyFollowers = "my_followers",
  MyFollowing = "my_following",
  Unfollow = "unfollow",
  LickMew = "lick_mew",
  UnlickMew = "unlick_mew",
  MyLicks = "my_licks",
  GetFeedMewAndContext = "get_feed_mew_and_context",
  GetMewsWithCashtag = "get_mews_with_cashtag",
  GetMewsWithHashtag = "get_mews_with_hashtag",
  GetMewsWithMention = "get_mews_with_mention",
  SearchCashtags = "search_cashtags",
  SearchHashtags = "search_hashtags",
}

export const callZome = async <T>(
  fnName: CallZomeRequest["fn_name"],
  payload: CallZomeRequest["payload"]
) => {
  const result: { type: "ok"; data: T } = await useClientStore().callZome({
    roleName: MEWSFEED_ROLE_NAME,
    zomeName: MEWS_ZOME_NAME,
    fnName,
    payload,
  });
  return result.data;
};

export const createMew = async (mew: CreateMewInput) =>
  callZome(MewsFn.CreateMew, mew);

export const getMew = async (mew: ActionHash): Promise<Mew> =>
  callZome(MewsFn.GetMew, mew);

export const mewsFeed = async (options: FeedOptions): Promise<Array<FeedMew>> =>
  callZome(MewsFn.MewsFeed, options);

export const basketFeed = async (
  input: RecommendedInput
): Promise<Array<TrustFeedMew>> => callZome(MewsFn.BasketFeed, input);

export const mewsBy = async (
  agent: AgentPubKey | AgentPubKeyB64
): Promise<Array<FeedMew>> =>
  callZome(
    MewsFn.MewsBy,
    typeof agent === "string" ? decodeHashFromBase64(agent) : agent
  );

export const follow = async (input): Promise<null> =>
  callZome(MewsFn.Follow, input);

export const unfollow = async (agent: AgentPubKey): Promise<null> =>
  callZome(MewsFn.Unfollow, agent);

export const followers = async (
  agent: AgentPubKey
): Promise<Array<AgentPubKey>> => callZome(MewsFn.Followers, agent);

export const following = async (
  agent: AgentPubKey
): Promise<Array<AgentPubKey>> => callZome(MewsFn.Following, agent);

export const myFollowers = async (): Promise<Array<AgentPubKey>> =>
  callZome(MewsFn.MyFollowers, null);

export const myFollowing = async (): Promise<Array<AgentPubKey>> =>
  callZome(MewsFn.MyFollowing, null);

export const lickMew = async (mew: ActionHash): Promise<null> =>
  callZome(MewsFn.LickMew, mew);

export const unlickMew = async (mew: ActionHash): Promise<null> =>
  callZome(MewsFn.UnlickMew, mew);

export const getFeedMewAndContext = async (
  mew_action_hash: ActionHash
): Promise<FeedMew> => callZome(MewsFn.GetFeedMewAndContext, mew_action_hash);

export const getMewsWithCashtag = async (cashtag: string): Promise<FeedMew[]> =>
  callZome(MewsFn.GetMewsWithCashtag, cashtag);

export const getMewsWithHashtag = async (hashtag: string): Promise<FeedMew[]> =>
  callZome(MewsFn.GetMewsWithHashtag, hashtag);

export const getMewsWithMention = async (
  agentPubKey: AgentPubKey
): Promise<FeedMew[]> => callZome(MewsFn.GetMewsWithMention, agentPubKey);

export const searchCashtags = async (query: string): Promise<string[]> =>
  callZome(MewsFn.SearchCashtags, query);

export const searchHashtags = async (query: string): Promise<string[]> =>
  callZome(MewsFn.SearchHashtags, query);
