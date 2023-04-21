import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
import { TAG_SYMBOLS } from "./utils/tags";
import HomePage from "./pages/HomePage.vue";
import MewsFeed from "./pages/MewsFeed.vue";
import BasketFeed from "./pages/BasketFeed.vue";
import MyProfile from "./pages/MyProfile.vue";
import AgentProfile from "./pages/AgentProfile.vue";
import MewYarn from "./pages/MewYarn.vue";
import TagMewsFeed from "./pages/TagMewsFeed.vue";
import NotFound from "./pages/NotFound.vue";

export const PATH = {
  [TAG_SYMBOLS.CASHTAG]: "cashtag",
  [TAG_SYMBOLS.HASHTAG]: "hashtag",
  [TAG_SYMBOLS.MENTION]: "handle",
};

export const ROUTES = {
  home: "home",
  myProfile: "myProfile",
  profiles: "profiles",
  feed: "feed",
  basket: "basket",
  yarn: "yarn",
  [PATH[TAG_SYMBOLS.CASHTAG]]: PATH[TAG_SYMBOLS.CASHTAG],
  [PATH[TAG_SYMBOLS.HASHTAG]]: PATH[TAG_SYMBOLS.HASHTAG],
  [PATH[TAG_SYMBOLS.MENTION]]: PATH[TAG_SYMBOLS.MENTION],
};

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: ROUTES.home,
    component: HomePage,
  },
  {
    path: "/my-profile/",
    name: ROUTES.myProfile,
    component: MyProfile,
  },
  {
    path: "/profiles/:agent",
    name: ROUTES.profiles,
    component: AgentProfile,
  },
  {
    path: "/feed",
    name: ROUTES.feed,
    component: MewsFeed,
  },
  {
    path: "/basket",
    name: ROUTES.basket,
    component: BasketFeed,
  },
  {
    path: "/yarn/:hash",
    name: ROUTES.yarn,
    component: MewYarn,
  },
  {
    path: `/${PATH[TAG_SYMBOLS.CASHTAG]}/:tag`,
    name: ROUTES[PATH[TAG_SYMBOLS.CASHTAG]],
    component: TagMewsFeed,
    meta: { tag: TAG_SYMBOLS.CASHTAG },
  },
  {
    path: `/${PATH[TAG_SYMBOLS.HASHTAG]}/:tag`,
    name: ROUTES[PATH[TAG_SYMBOLS.HASHTAG]],
    component: TagMewsFeed,
    meta: { tag: TAG_SYMBOLS.HASHTAG },
  },
  {
    path: `/${PATH[TAG_SYMBOLS.MENTION]}/:tag/:agentPubKey`,
    name: ROUTES[PATH[TAG_SYMBOLS.MENTION]],
    component: TagMewsFeed,
    meta: { tag: TAG_SYMBOLS.MENTION },
  },
  { path: "/:pathMatch(.*)", component: NotFound },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
