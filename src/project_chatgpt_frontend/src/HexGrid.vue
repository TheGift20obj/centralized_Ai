<template>
  <div 
    class="grid" 
    :style="{
      gridTemplateColumns: `repeat(${gridCols}, 20px)`,
      gridTemplateRows: `repeat(${gridRows}, 20px)`
    }"
  >
    <div v-for="(color, i) in hexColors" 
         :key="i"
         class="cell"
         :style="{ backgroundColor: color }"></div>
  </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  content: String,
  gridCols: { type: Number, default: 16 },
  gridRows: { type: Number, default: 16 }
});

const hexColors = computed(() => {
  const HEX_RE = /#[0-9a-fA-F]{6}/g;
  return props.content.match(HEX_RE) || [];
});
</script>

<style scoped>
.grid {
  display: grid;
  gap: 0px;
  margin-top: 10px;
}
.cell {
  width: 20px;
  height: 20px;
  border: 0px solid #ddd;
}
</style>