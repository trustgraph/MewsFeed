<template>
  <q-card square class="q-mb-md text-body1">
    <q-checkbox v-model="allTopics" label="Follow All Topics" />

    <!-- <div class="flex items-center justify-between" v-if="!allTopics">
      <div class="flex" style="font-weight: bold">topic</div>
      <div class="flex" style="font-weight: bold">weight</div>
    </div> -->

    <q-list v-if="!allTopics">
      <TrustGraphAtom
        v-for="(atom, i) in atoms"
        :key="i"
        :value="atom"
        @input="(val) => atoms.splice(i, 1, val)"
      />
    </q-list>
    <button @click="addAtom">Add</button>
  </q-card>
</template>

<script setup lang="ts">
import TrustGraphAtom from "@/components/TrustGraphAtom.vue";
import { onMounted, ref } from "vue";
import { useQuasar, QCheckbox } from "quasar";
import { TrustGraphAtomData } from "../types/types";

const $q = useQuasar();

defineEmits(["update:topic", "update:weight"]);

const atoms = ref<TrustGraphAtomData[]>([]);

const allTopics = ref(true);

const addAtom = async () => {
  atoms.value.push({ topic: "", weight: 0 });
};
</script>
