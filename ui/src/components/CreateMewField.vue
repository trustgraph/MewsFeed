<template>
  <div class="text-center">
    <div ref="mewContainer" class="text-left" style="position: relative">
      <q-card
        contenteditable="true"
        class="mew-content text-body1 q-pa-md overflow-auto"
        style="word-break: break-all"
        bordered
        flat
        @keydown="onKeyDown"
        @keyup="onKeyUp"
        @mouseup="onMouseUp"
        @paste="onPaste"
        @input="onInput"
      />
      <div class="flex justify-between q-pa-sm">
        <div>
          <div
            :class="{
              'text-red text-bold':
                isMewFull || isMewRequireTruncation || isMewOverfull,
              'text-caption text-grey': !isMewFull && !isMewOverfull,
            }"
          >
            {{ mewContentLength }} /
            {{ min([TRUNCATED_MEW_LENGTH, mewLengthMax]) }} Characters
          </div>
          <div style="height: 20px">
            <div v-if="isMewUnderfull" class="text-caption text-grey">
              {{ mewLengthMin }} Minimum
            </div>
            <div v-if="isMewRequireTruncation" class="text-caption text-grey">
              Overflow will be hidden
            </div>
          </div>
        </div>

        <div class="q-mb-xs text-right text-caption text-grey">
          Ctrl/Cmd + Enter to publish
        </div>
      </div>

      <q-card
        class="link-target-input-container q-px-md q-py-sm"
        style="min-width: 13rem"
      >
        <q-input
          ref="linkTargetInput"
          v-model="linkTarget"
          type="text"
          placeholder="Paste a URL to create a link"
          dense
          borderless
          :rules="[(val: string) => isRawUrl(val) || 'Link target must be valid URL']"
          @keydown.enter="createLinkTag"
          @keydown.space="createLinkTag"
          @keydown.tab="createLinkTag"
          @blur="resetLinkTargetInput"
        />
      </q-card>

      <q-card class="autocompleter">
        <template v-if="currentAgentSearch.length < 3">
          <q-card-section>Min. 3 letters</q-card-section>
        </template>

        <template v-else>
          <q-spinner-pie
            v-if="autocompleterLoading"
            color="secondary"
            size="md"
            class="q-mx-lg q-my-xs"
          />

          <q-list
            v-else
            class="autocompleter-list bg-white rounded-borders"
            bordered
            dense
            separator
            tabindex="-1"
          >
            <q-item
              v-for="([agentPubKey, profile], i) in agentAutocompletions"
              :key="i"
              clickable
              @keydown="onAutocompleteKeyDown"
              @click="onAutocompleteAgentSelect(agentPubKey, profile)"
            >
              <q-item-section avatar class="q-pr-sm col-shrink">
                <agent-avatar
                  :agentPubKey="agentPubKey"
                  disable-tooltip
                  disable-copy
                  size="30"
                ></agent-avatar>
              </q-item-section>
              <q-item-section>
                {{ profile.fields[PROFILE_FIELDS.DISPLAY_NAME] }}
                {{ TAG_SYMBOLS.MENTION }}{{ profile.nickname }}
              </q-item-section>
            </q-item>

            <q-item v-if="agentAutocompletions.length === 0">
              <q-item-section>Nothing found, Kitty</q-item-section>
            </q-item>
          </q-list>
        </template>
      </q-card>

      <q-icon name="help" color="grey" size="xs" class="help-text">
        <q-tooltip
          class="text-body2"
          anchor="top middle"
          self="bottom middle"
          :delay="TOOLTIP_DELAY"
        >
          You can mention people with @ and use #hashtags and $cashtags as well
          as ^links in a mew.
        </q-tooltip>
      </q-icon>
    </div>

    <q-btn
      :disable="isMewEmpty || isMewOverfull || isMewUnderfull"
      :loading="saving"
      :tabindex="agentAutocompletions.length && 0"
      color="accent"
      @click="publishMew"
    >
      Publish Mew
    </q-btn>
  </div>
</template>

<script setup lang="ts">
import { useClientStore, useMewsfeedStore } from "@/stores";
import { showError } from "@/utils/notification";
import { useSearchProfiles, useMyProfile } from "@/utils/profile";
import { Profile } from "@holochain-open-dev/profiles";
import { isMentionTag, isRawUrl, isLinkTag, TAG_SYMBOLS } from "@/utils/tags";
import { onMounted, PropType, ref, computed } from "vue";
import {
  CreateMewInput,
  ElementWithInnerText,
  LinkTarget,
  LinkTargetName,
  MewsfeedDnaProperties,
  MewType,
  PROFILE_FIELDS,
  TOOLTIP_DELAY,
} from "../types/types";
import { AgentPubKey } from "@holochain/client";
import min from "lodash.min";
import { decode } from "@msgpack/msgpack";
import { union, flatten } from "lodash";

const ANCHOR_DATA_ID_AGENT_PUB_KEY = "agentPubKey";
const ANCHOR_DATA_ID_URL = "url";
const POPUP_MARGIN_TOP = 20;
let currentAnchorOffset: number;
let currentFocusOffset: number;
let currentNode: Node;

const emit = defineEmits<{ (e: "publish-mew"): void }>();

const props = defineProps({
  mewType: { type: Object as PropType<MewType>, required: true },
});

const store = useMewsfeedStore();
const clientStore = useClientStore();

const { searchProfiles } = useSearchProfiles();
const { runWhenMyProfileExists } = useMyProfile();

const TRUNCATED_MEW_LENGTH = 300;

const mewContainer = ref<HTMLDivElement | null>(null);
const mewContentLength = ref(0);
const saving = ref(false);
const linkTarget = ref();
const linkTargetInput = ref();
const currentAgentSearch = ref("");
const agentAutocompletions = ref<Array<[AgentPubKey, Profile]>>([]);
const autocompleterLoading = ref(false);
const mewLengthMin = ref();
const mewLengthMax = ref();

const isMewEmpty = computed(() => mewContentLength.value === 0);
const isMewFull = computed(
  () => mewLengthMin.value && mewContentLength.value === mewLengthMax.value
);
const isMewRequireTruncation = computed(
  () => mewContentLength.value > TRUNCATED_MEW_LENGTH
);
const isMewOverfull = computed(
  () => mewLengthMax.value && mewContentLength.value > mewLengthMax.value
);
const isMewUnderfull = computed(
  () => mewLengthMin.value && mewContentLength.value < mewLengthMin.value
);

onMounted(async () => {
  setTimeout(focusInputField, 0);

  const appInfo = await clientStore.appInfo();
  const dnaProperties = decode(
    appInfo.cell_info.mewsfeed[0].provisioned.dna_modifiers.properties
  ) as MewsfeedDnaProperties;

  mewLengthMin.value = dnaProperties.mew_characters_min;
  mewLengthMax.value = dnaProperties.mew_characters_max;
});

const collectLinksWithinElement = (element: Element): LinkTarget[] => {
  if (element.children.length > 0) {
    console.log("element has children", element.children);
    const childrenLinks: LinkTarget[] = Array.from(element.children)
      .map((child) => {
        console.log(
          "child info",
          child,
          child.tagName,
          child instanceof HTMLAnchorElement
        );

        if (
          child &&
          child.tagName === "A" &&
          child instanceof HTMLAnchorElement
        ) {
          console.log("child is link");
          if (child.dataset[ANCHOR_DATA_ID_AGENT_PUB_KEY]) {
            const agentPubKeyString =
              child.dataset[ANCHOR_DATA_ID_AGENT_PUB_KEY];
            const agentPubKey = Uint8Array.from(
              agentPubKeyString.split(",") as Iterable<number>
            );

            return { [LinkTargetName.Mention]: agentPubKey } as LinkTarget;
          } else if (
            (element as HTMLAnchorElement).dataset[ANCHOR_DATA_ID_URL]
          ) {
            const url = (element as HTMLAnchorElement).dataset[
              ANCHOR_DATA_ID_URL
            ];

            return { [LinkTargetName.URL]: url } as LinkTarget;
          }
        }
      })
      .filter((l) => l !== undefined)
      .map((l) => l as LinkTarget);

    return union(
      childrenLinks,
      flatten(
        Array.from(element.children).map((child) =>
          collectLinksWithinElement(child)
        )
      )
    );
  } else {
    return [];
  }
};

const publishMew = () => {
  runWhenMyProfileExists(async () => {
    const mewInput = mewContainer.value?.querySelector(
      ".mew-content"
    ) as ElementWithInnerText;
    if (!mewInput) return;

    // build link array
    const links = collectLinksWithinElement(mewInput);

    const createMewInput: CreateMewInput = {
      mewType: props.mewType,
      text: getTrimmedText(),
      links: links.length ? links : undefined,
    };
    try {
      saving.value = true;
      await store.createMew(createMewInput);
    } catch (error) {
      showError(error);
    } finally {
      saving.value = false;
    }
    emit("publish-mew");
    mewInput.textContent = "";
    mewContentLength.value = 0;
    hideAutocompleter();
    resetLinkTargetInput();
    focusInputField();
  });
};

/**
 * Link Tag Handling
 */
const createLinkTag = (e: Event) => {
  if (!isRawUrl(linkTarget.value)) return;
  e.preventDefault();

  // Extract label text
  const range = new Range();
  range.setStart(currentNode, currentAnchorOffset);
  range.setEnd(currentNode, currentFocusOffset);
  const label = range.extractContents().textContent;

  // Create html link element
  const anchor = document.createElement("a");
  anchor.textContent = label;
  anchor.href = "#";
  anchor.dataset[ANCHOR_DATA_ID_URL] = linkTarget.value;
  range.insertNode(anchor);

  // insert space after html link
  const spaceNode = document.createTextNode(String.fromCharCode(160));
  anchor.after(spaceNode);

  // reset input
  resetLinkTargetInput();
  document.getSelection()?.setPosition(spaceNode, 1);
};

/**
 * Profile Tag Handling
 */
const loadAutocompleterUsers = async (nickname: string) => {
  try {
    autocompleterLoading.value = true;
    agentAutocompletions.value = await searchProfiles(nickname);
  } catch (error) {
    showError(error);
  } finally {
    autocompleterLoading.value = false;
  }
};

const onAutocompleteKeyDown = (keyDownEvent: KeyboardEvent) => {
  if (keyDownEvent.key === "ArrowDown") {
    keyDownEvent.preventDefault();
    const currentListItem = keyDownEvent.currentTarget;
    if (currentListItem instanceof HTMLElement) {
      const nextSibling = currentListItem.nextSibling;
      if (nextSibling instanceof HTMLElement) {
        nextSibling.focus();
      }
    }
  } else if (keyDownEvent.key === "ArrowUp") {
    keyDownEvent.preventDefault();
    const currentListItem = keyDownEvent.currentTarget;
    if (currentListItem instanceof HTMLElement) {
      const previousSibling = currentListItem.previousSibling;
      if (previousSibling instanceof HTMLElement) {
        previousSibling.focus();
      } else {
        const mewContent = mewContainer.value?.querySelector(".mew-content");
        if (mewContent instanceof HTMLElement) {
          mewContent.focus();
        }
      }
    }
  }
};

const onAutocompleteAgentSelect = (agent: AgentPubKey, profile: Profile) => {
  const range = new Range();
  range.setStart(currentNode, currentAnchorOffset);
  range.setEnd(currentNode, currentFocusOffset);

  // Insert mention link: '@' + agent username
  const anchor = document.createElement("a");
  anchor.href = "#";
  anchor.textContent = TAG_SYMBOLS.MENTION + profile.nickname;
  anchor.dataset[ANCHOR_DATA_ID_AGENT_PUB_KEY] = agent.toString();
  range.deleteContents();
  range.insertNode(anchor);

  // insert space after mention
  const spaceNode = document.createTextNode(String.fromCharCode(160));
  anchor.after(spaceNode);

  // reset autocomplete input
  hideAutocompleter();
  document.getSelection()?.setPosition(spaceNode, 1);

  setMewContentLength();
};

/**
 * Text Input Handling
 */

const onInput = (event: KeyboardEvent) => {
  setMewContentLength();

  // Prevent input of characters if mew is already full
  if (
    (isMewFull.value || isMewOverfull.value) &&
    event.key !== "Backspace" &&
    event.key !== "Delete" &&
    !(event.key === "a" && event.ctrlKey) &&
    event.key !== "ArrowLeft" &&
    event.key !== "ArrowRight" &&
    event.key !== "ArrowUp" &&
    event.key !== "ArrowDown"
  ) {
    event.preventDefault();
  }
};

const onKeyDown = (event: KeyboardEvent) => {
  setMewContentLength();

  // Support Meta + Enter keys to publish
  if (
    event.key === "Enter" &&
    event.metaKey &&
    !(isMewEmpty.value || isMewOverfull.value)
  ) {
    publishMew();
  }

  // Support KeyDown or Tab keys to focus on first item displayed in agents list (after typing an agent tag)
  else if (event.key === "ArrowDown" || event.key === "Tab") {
    const firstListItem = mewContainer.value?.querySelector(".q-item");
    if (firstListItem instanceof HTMLElement) {
      event.preventDefault();
      firstListItem.focus();
    }
  }

  // Prevent input of leading line breaks
  // Prevent input of trailing line breaks if mew is already full
  else if (
    event.key === "Enter" &&
    (isMewEmpty.value || isMewFull.value || isMewOverfull.value)
  ) {
    event.preventDefault();
  }

  // Prevent input of more than 3 consecutive line breaks
  else if (
    event.key === "Enter" &&
    !(isMewEmpty.value || isMewOverfull.value)
  ) {
    const textContent = getRawText();

    if (textContent.slice(-3) === "\n\n\n") {
      event.preventDefault();
    }
  } else {
    onInput(event);
  }
};

const onKeyUp = (keyUpEvent: KeyboardEvent) => {
  if (!(keyUpEvent.currentTarget instanceof HTMLElement)) {
    return;
  }
  const content = keyUpEvent.currentTarget.textContent;
  const selection = document.getSelection();
  if (keyUpEvent.key === " ") {
    hideAutocompleter();
    resetLinkTargetInput();
  } else if (
    (keyUpEvent.key === "Backspace" || keyUpEvent.key === "Delete") &&
    selection?.anchorNode?.parentElement?.tagName === "A"
  ) {
    stripAnchorFromLink(selection);
  } else if (keyUpEvent.key.length === 1 && content) {
    // all single characters
    onCaretPositionChange();
  }
};

const onMouseUp = () => {
  hideAutocompleter();
  resetLinkTargetInput();
};

const onPaste = (event: ClipboardEvent) => {
  event.preventDefault();
  const data = event.clipboardData?.getData("text/plain");

  if (data) {
    const pastedNode = document.createTextNode(data);
    document.getSelection()?.getRangeAt(0).insertNode(pastedNode);
    document.getSelection()?.setPosition(pastedNode, pastedNode.length);
    onCaretPositionChange();

    setMewContentLength();
  }
};

const onCaretPositionChange = () => {
  const selection = document.getSelection();
  const content = selection?.anchorNode?.textContent;
  if (!selection || !content) {
    return;
  }

  // find end of word that the caret is positioned at
  const ahead = content.substring(selection.anchorOffset);
  const nextWhitespace = /\s/.exec(ahead);
  const endOfAheadIndex = nextWhitespace ? nextWhitespace.index : ahead.length;
  const endOfWordIndex = selection.anchorOffset + endOfAheadIndex;

  // find start of word that the caret is positioned at
  const behind = content.substring(0, selection.anchorOffset - 1);
  let lastSpaceIndex = -1;
  // find last index of space, which can be " " (32) or "&nbsp;" (160)
  for (let i = behind.length - 1; i >= 0; i--) {
    if (behind.charCodeAt(i) === 32 || behind.charCodeAt(i) === 160) {
      lastSpaceIndex = i;
      break;
    }
  }

  const startOfWordIndex = lastSpaceIndex === -1 ? 0 : lastSpaceIndex + 1;
  const currentWord = content.substring(startOfWordIndex, endOfWordIndex);

  if (currentWord.length && selection.anchorNode) {
    // current word starts with @ and is followed by at least another word character
    if (isMentionTag(currentWord)) {
      showElement(selection.anchorNode, startOfWordIndex, ".autocompleter");

      const nicknameChars = currentWord.substring(1);
      currentAgentSearch.value = nicknameChars;
      if (nicknameChars.length >= 3) {
        loadAutocompleterUsers(nicknameChars);
        currentAnchorOffset = startOfWordIndex;
        currentFocusOffset = endOfWordIndex;
      }
      // current word is a URL
    } else if (isLinkTag(currentWord)) {
      showElement(
        selection.anchorNode,
        startOfWordIndex,
        ".link-target-input-container"
      );
      currentAnchorOffset = startOfWordIndex;
      currentFocusOffset = endOfWordIndex;
    } else {
      hideAutocompleter();
      resetLinkTargetInput();
    }
  }
};

/**
 * Helper Utils
 */

const focusInputField = () =>
  document
    .getSelection()
    ?.setPosition(mewContainer.value?.querySelector(".mew-content") || null, 0);

const stripAnchorFromLink = (selection: Selection) => {
  if (!selection.anchorNode?.parentElement) {
    return;
  }
  const parentElement = selection.anchorNode.parentElement;
  const linkContent = parentElement.firstChild?.cloneNode();
  if (linkContent) {
    const offset = selection.anchorOffset;
    parentElement.after(linkContent);
    parentElement.remove();
    selection.setPosition(linkContent, offset);
  }
};

const getRawText = (): string => {
  const text = (
    mewContainer.value?.querySelector(
      ".mew-content"
    ) as null | ElementWithInnerText
  )?.innerText;

  return text ? text : "";
};

const getTrimmedText = (): string => {
  const text = getRawText();
  return text ? text.trim() : "";
};

const setMewContentLength = () => {
  const text = getTrimmedText();
  mewContentLength.value = text.length;
};

const showElement = (
  anchorNode: Node,
  startOfWordIndex: number,
  selector: string
) => {
  const range = new Range();
  range.setStart(anchorNode, startOfWordIndex);
  currentNode = anchorNode;
  const selectionRect = range.getBoundingClientRect();
  const element = mewContainer.value?.querySelector(selector);
  if (element instanceof HTMLElement && mewContainer.value) {
    element.style.top =
      Math.max(
        0,
        selectionRect.top - mewContainer.value.getBoundingClientRect().top
      ) +
      POPUP_MARGIN_TOP +
      "px";
    element.style.left =
      Math.max(
        0,
        selectionRect.left - mewContainer.value.getBoundingClientRect().left
      ) + "px";
    element.style.display = "block";
  }
};

const hideElement = (selector: string) => {
  const element = mewContainer.value?.querySelector(selector);
  if (element && element instanceof HTMLElement) {
    element.style.display = "none";
  }
};

const resetLinkTargetInput = () => {
  hideElement(".link-target-input-container");
  linkTargetInput.value.resetValidation();
  linkTarget.value = undefined;
};

const hideAutocompleter = hideElement.bind(null, ".autocompleter");
</script>

<style lang="sass">
.mew-content
  min-height: $body-font-size * $body-line-height * 2

  &:focus
    outline-color: $primary
  a
    color: $secondary
    font-weight: 600

.help-text
  position: absolute
  cursor: default
  top: 5px
  right: 5px

.autocompleter, .link-target-input-container
  position: absolute
  display: none
  z-index: 1
</style>
