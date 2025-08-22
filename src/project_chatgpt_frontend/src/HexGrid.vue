<template>
  <div>
    <!-- Canvas gdy nie ma trybu grid -->
    <canvas 
      v-show="!showGrid" 
      ref="canvas" 
      :width="canvasWidth" 
      :height="canvasHeight">
    </canvas>

    <div 
      v-show="showGrid" 
      class="grid"
      :style="{
        display: 'grid',
        gridTemplateColumns: `repeat(${gridCols}, ${cellSize}px)`,
        gridTemplateRows: `repeat(${gridRows}, ${cellSize}px)`,
        gap: '1px'
      }"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
    >
      <template v-for="r in gridRows" :key="r">
        <button
          v-for="c in gridCols"
          :key="`${r}-${c}`"
          class="cell-btn"
          :class="{ selected: isInSelection(c-1, r-1) }"
          :style="{
            width: cellSize + 'px',
            height: cellSize + 'px',
            backgroundColor: getCellColor(c-1, r-1)
          }"
          @click="gridMode === 'cell' ? selectCell(c-1, r-1) : null"
        ></button>
      </template>
    </div>

    <div style="margin-top: 8px;">
      <button @click="toggleGrid">{{ showGrid ? 'Canvas View' : 'Grid View' }}</button>
      <button @click="downloadImage" v-if="!showGrid">Download</button>
      <button v-if="showGrid" @click="toggleGridMode">
        {{ gridMode === 'cell' ? 'Selection Mode' : 'Cell Mode' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, nextTick } from "vue";

const isSelecting = ref(false);
const selectionStart = ref(null);
const selectionEnd = ref(null);
const gridMode = ref("cell");

const toggleGridMode = () => {
  gridMode.value = gridMode.value === "cell" ? "selection" : "cell";
};

const startSelection = (e) => {
  if (e.button !== 0) return; // tylko lewy klik
  isSelecting.value = true;

  const pos = getCellFromMouse(e);
  selectionStart.value = pos;
  selectionEnd.value = pos;
};

const updateSelection = (e) => {
  if (!isSelecting.value) return;
  selectionEnd.value = getCellFromMouse(e);
};

const emit = defineEmits(["selectCell", "selectArea"]);

const endSelection = () => {
  if (!isSelecting.value) return;
  isSelecting.value = false;

  if (selectionStart.value && selectionEnd.value) {
    const rect = normalizeRect(selectionStart.value, selectionEnd.value);
    const { x1, y1, x2, y2 } = rect;
    const r_x1 = Math.floor((canvasWidth - (x1 * props.cellSize + props.cellSize)) / props.cellSize);
    const r_y1 = Math.floor((canvasHeight - (y1 * props.cellSize + props.cellSize)) / props.cellSize);
    const r_x2 = Math.floor((canvasWidth - (x2 * props.cellSize + props.cellSize)) / props.cellSize);
    const r_y2 = Math.floor((canvasHeight - (y2 * props.cellSize + props.cellSize)) / props.cellSize);
    const rev_rect = { x1: r_x1, y1: r_y1, x2: r_x2, y2: r_y2 };
    emit("selectArea", rev_rect); // wyślij do parenta
  }
};

const getCellFromMouse = (e) => {
  const gridRect = e.currentTarget.getBoundingClientRect();
  const c = Math.floor((e.clientX - gridRect.left) / (props.cellSize + 1));
  const r = Math.floor((e.clientY - gridRect.top) / (props.cellSize + 1));
  return { c, r };
};

const onMouseDown = (e) => {
  if (gridMode.value === "selection") startSelection(e);
};

const onMouseMove = (e) => {
  if (gridMode.value === "selection") updateSelection(e);
};

const onMouseUp = (e) => {
  if (gridMode.value === "selection") endSelection(e);
};

const normalizeRect = (p1, p2) => ({
  x1: Math.min(p1.c, p2.c),
  y1: Math.min(p1.r, p2.r),
  x2: Math.max(p1.c, p2.c),
  y2: Math.max(p1.r, p2.r),
});

const isInSelection = (c, r) => {
  if (!selectionStart.value || !selectionEnd.value) return false;
  const { x1, y1, x2, y2 } = normalizeRect(selectionStart.value, selectionEnd.value);
  return c >= x1 && c <= x2 && r >= y1 && r <= y2;
};

const props = defineProps({
  content: String,
  gridCols: { type: Number, default: 16 },
  gridRows: { type: Number, default: 16 },
  cellSize: { type: Number, default: 3 },
});

const canvas = ref(null);
const canvasWidth = props.gridCols * props.cellSize;
const canvasHeight = props.gridRows * props.cellSize;

const showGrid = ref(false);

const hexColors = () => {
  const HEX_RE = /#[0-9a-fA-F]{6}/g;
  return props.content.match(HEX_RE) || [];
};

const drawGrid = () => {
  if (!canvas.value) return;
  const ctx = canvas.value.getContext("2d");
  const colors = hexColors();
  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  for (let i = 0; i < colors.length; i++) {
    let x = (i % props.gridCols) * props.cellSize;
    let y = Math.floor(i / props.gridCols) * props.cellSize;

    // odbicie względem osi X i Y
    x = canvasWidth - (x + props.cellSize);
    y = canvasHeight - (y + props.cellSize);

    ctx.fillStyle = colors[i];
    ctx.fillRect(x, y, props.cellSize, props.cellSize);
  }
};

const toggleGrid = async () => {
  showGrid.value = !showGrid.value;

  if (!showGrid.value) {
    // poczekaj aż canvas wróci do DOM
    await nextTick();
    drawGrid();
  }
};

// 👉 funkcja do pobierania koloru dla przycisku w gridzie
const getCellColor = (c, r) => {
  const colors = hexColors();

  // indeks w tablicy kolorów (tak jak normalnie bez odbicia)
  const i = r * props.gridCols + c;

  if (i >= colors.length) return "transparent";

  // obliczamy odbite współrzędne
  let x = c * props.cellSize;
  let y = r * props.cellSize;

  x = canvasWidth - (x + props.cellSize);
  y = canvasHeight - (y + props.cellSize);

  // obliczamy index odbity
  const mirroredCol = Math.floor(x / props.cellSize);
  const mirroredRow = Math.floor(y / props.cellSize);
  const mirroredIndex = mirroredRow * props.gridCols + mirroredCol;

  return colors[mirroredIndex] || "transparent";
};

// 👉 kliknięcie w przycisk
const selectCell = (c, r) => {
  // odbicie współrzędnych
  let x = c * props.cellSize;
  let y = r * props.cellSize;

  x = canvasWidth - (x + props.cellSize);
  y = canvasHeight - (y + props.cellSize);

  const mirroredCol = Math.floor(x / props.cellSize);
  const mirroredRow = Math.floor(y / props.cellSize);

  emit("selectCell", { x: mirroredCol, y: mirroredRow });
};

const downloadImage = () => {
  const link = document.createElement("a");
  link.download = "image.png";
  link.href = canvas.value.toDataURL("image/png");
  link.click();
};

onMounted(drawGrid);
watch(() => props.content, drawGrid);
</script>

<style scoped>
.cell-btn.selected {
  outline: 2px solid blue;
}
.cell-btn {
  border: 1px solid rgba(0,0,0,0.1);
  padding: 0;
  cursor: pointer;
}
.cell-btn:hover {
  filter: brightness(80%);
}
</style>
