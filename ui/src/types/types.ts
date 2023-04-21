import {
  ActionHash,
  AgentPubKey,
  Create,
  HoloHash,
  SigningCredentials,
} from "@holochain/client";
import { RouteLocationNamedRaw } from "vue-router";

export const PROFILE_FIELDS = {
  DISPLAY_NAME: "Display name",
  BIO: "Bio",
  LOCATION: "Location",
};

export const TOOLTIP_DELAY = 400;

export enum LinkTargetName {
  Mention = "Mention",
  URL = "URL",
}

export interface MentionLinkTarget {
  [LinkTargetName.Mention]: AgentPubKey;
}

export interface UrlLinkTarget {
  [LinkTargetName.URL]: string;
}

export type LinkTarget = MentionLinkTarget | UrlLinkTarget;

export type CreateMewInput = {
  mewType: MewType;
  text: string | null;
  links?: LinkTarget[];
};

export interface MewContent {
  text: string;
  links?: LinkTarget[];
}

export enum MewTagType {
  Mention,
  Link,
  RawUrl,
  Cashtag,
  Hashtag,
}

export interface MewContentPart {
  text: string;
  route?: RouteLocationNamedRaw;
  href?: string;
  tagType?: MewTagType;
}

export enum MewTypeName {
  Original = "original",
  Reply = "reply",
  MewMew = "mewMew",
  Quote = "quote",
}

export type MewType =
  | {
      [MewTypeName.Original]: null;
    }
  | {
      [MewTypeName.Reply]: ActionHash;
    }
  | {
      [MewTypeName.MewMew]: ActionHash;
    }
  | {
      [MewTypeName.Quote]: ActionHash;
    };

export interface Mew {
  mewType: MewType;
  content: MewContent | null;
}

export interface FollowInput {
  agent: AgentPubKey;
  followTopics: FollowTopicInput[];
  followOther: boolean;
}

export interface FollowTopicInput {
  topic: string;
  weight: string;
}

export interface FeedMew {
  mew: Mew;
  action: Create;
  actionHash: ActionHash;
  replies: HoloHash[];
  quotes: HoloHash[];
  licks: AgentPubKey[];
  mewmews: HoloHash[];
}

export interface RecommendedInput {
  now: number; // microseconds since epoch
  oldestMewSeconds: number | null;
}

export interface TrustFeedMew {
  feedMew: FeedMew;
  weight: number;
  topic: string | null;
}

export interface FeedOptions {
  option: string;
}

export interface NotificationOptions {
  color?: string;
  textColor?: string;
  message?: string;
  caption?: string;
  html?: boolean;
  icon?: string;
  position?:
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right"
    | "top"
    | "bottom"
    | "left"
    | "right"
    | "center";
  actions?: Array<() => void>;
  onDismiss?: () => void;
}

export enum SearchResult {
  Agent,
  Hashtag,
  Cashtag,
}

export interface ElementWithInnerText extends Element {
  innerText: string;
}

export interface SigningCredentialsJson
  extends Omit<SigningCredentials, "capSecret" | "keyPair" | "signingKey"> {
  capSecret: number[];
  keyPair: {
    publicKey: number[];
    secretKey: number[];
  };
  signingKey: number[];
}

export interface MewsfeedDnaProperties {
  mew_characters_min?: number;
  mew_characters_max?: number;
}

export interface TrustGraphAtomData {
  topic: string;
  weight: number;
}
