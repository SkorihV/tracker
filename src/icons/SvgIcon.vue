<script setup>
import { computed } from "vue";

const props = defineProps({
  name: { type: String, required: true },
});

const icons = import.meta.glob("../icons/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

const svg = computed(() => {
  const key = Object.keys(icons).find((k) => k.endsWith(`/${props.name}.svg`));
  return key ? icons[key] : "";
});
</script>

<template>
  <span class="svg-icon" aria-hidden="true" v-html="svg" />
</template>

<style scoped>
.svg-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.1em;
  height: 1.1em;
  line-height: 0;
}
.svg-icon :deep(svg) {
  width: 1em;
  height: 1em;
}
</style>