<template>
  <div>
    <canvas ref="canvas" :width="canvasWidth" :height="canvasHeight"></canvas>
    <div style="margin-top: 8px;">
      <button @click="toggleGrid">{{ showGrid ? 'Hide Grid' : 'Show Grid' }}</button>
      <button @click="downloadImage">Download</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch } from "vue";

const props = defineProps({
  content: String,
  gridCols: { type: Number, default: 16 },
  gridRows: { type: Number, default: 16 },
  cellSize: { type: Number, default: 8 }
});

const canvas = ref(null);
const canvasWidth = props.gridCols * props.cellSize;
const canvasHeight = props.gridRows * props.cellSize;

const showGrid = ref(false); // reactive do przełączania siatki

const hexColors = () => {
  const HEX_RE = /#[0-9a-fA-F]{6}/g;
  return props.content.match(HEX_RE) || [];
};

const drawGrid = () => {
  const ctx = canvas.value.getContext("2d");
  const colors = hexColors();
  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  // Rysowanie pikseli
  for (let i = 0; i < colors.length; i++) {
    let x = (i % props.gridCols) * props.cellSize;
    let y = Math.floor(i / props.gridCols) * props.cellSize;

    // Odbicie względem osi X i Y
    x = canvasWidth - (x + props.cellSize);
    y = canvasHeight - (y + props.cellSize);

    ctx.fillStyle = colors[i];
    ctx.fillRect(x, y, props.cellSize, props.cellSize);
  }

  // Rysowanie siatki jeśli showGrid
  if (showGrid.value) {
    ctx.strokeStyle = "rgba(0,0,0,0.3)";
    ctx.lineWidth = 1;

    // pionowe linie
    for (let c = 0; c <= props.gridCols; c++) {
      const x = c * props.cellSize;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvasHeight);
      ctx.stroke();
    }

    // poziome linie
    for (let r = 0; r <= props.gridRows; r++) {
      const y = r * props.cellSize;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvasWidth, y);
      ctx.stroke();
    }
  }
};

const toggleGrid = () => {
  showGrid.value = !showGrid.value;
  drawGrid(); // odświeżenie canvasu
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