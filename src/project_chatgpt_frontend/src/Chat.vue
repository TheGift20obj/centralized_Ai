<template>
  <!-- Chat main area -->
  <div class="flex flex-col h-screen bg-gray-950 text-white">
    <!-- Messages area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3">
      <div
          v-for="(msg, index) in messages"
          :key="index"
          class="message-block"
        >
          <!-- Date -->
          <div class="message-date">
            {{ new Date(Number(msg.etc[0])).toLocaleDateString([], { year: 'numeric', month: '2-digit', day: '2-digit' }) }}
          </div>

          <!-- User -->
          <div v-if="msg.role === 'user'" class="message-wrapper user-wrapper">
            <div class="message-author">{{ loginStatus.username }}</div>
            <div class="message user-message">
              <HexGrid v-if="hasHex(msg.content)" :content="msg.content" :grid-cols="msg.etc[1]" :grid-rows="msg.etc[2]" @selectCell="handleSelectCell" @selectArea="handleSelectRect" :style="{ aspectRatio: msg.etc[1] + ' / ' + msg.etc[2] }"/>
              <span v-else>{{ msg.content }}</span>
              <div v-if="selectedModel.includes('Image') && hasHex(msg.content)">
                <div v-if="selectedMsg === index">
                  <span>Selected</span>
                </div>
                <div v-else>
                  <button @click="select_image(index)">Select</button>
                </div>
              </div>
            </div>
            <div class="message-time">
              {{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
            </div>
          </div>

          <!-- AI -->
          <div v-else class="message-wrapper ai-wrapper">
            <div class="message-author">{{ msg.role }}</div>
            <div class="message ai-message">
              <span v-html="formatMarkdown(msg.content)"></span>
            </div>
            <div class="message-time">
              {{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
            </div>
          </div>
        </div>
        <div ref="endOfMessages"></div>
    </div>

    <!-- Composer -->
    <div class="border-t border-gray-800 p-3 flex items-center gap-2">

      <!-- Model selector -->
      <select
        v-model="selectedModel"
        class="bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
      >
        <option v-for="model in models" :key="model" :value="model">
          {{ model }}
        </option>
      </select>

      <!-- Jeśli model NIE kończy się na "Image" -->
      <template v-if="!selectedModel.endsWith('Image')">
        <!-- Input -->
        <textarea
          v-model="newMessage"
          rows="1"
          placeholder="Type your message..."
          class="flex-1 resize-none bg-gray-900 border border-gray-700 rounded p-2 focus:outline-none"
          @keydown.enter.prevent="sendMessage"
        ></textarea>

        <!-- Mic button -->
        <button
          class="px-3 py-2 bg-gray-800 hover:bg-gray-700 rounded"
          @click="micInput"
        >
          🎤
        </button>

        <!-- Send button -->
        <button
          class="px-3 py-2 bg-blue-600 hover:bg-blue-500 rounded"
          @click="sendMessage"
          :disabled="!loginStatus.loggedIn || !newMessage.trim()"
        >
          ➤
        </button>
      </template>

      <!-- Jeśli model kończy się na "Image" -->
      <template v-else>
        <!-- Inputs -->
        <input
          v-model="imageParams.cols"
          type="number"
          placeholder="Cols"
          class="w-16 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />
        <input
          v-model="imageParams.rows"
          type="number"
          placeholder="Rows"
          class="w-16 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />
        <input
          v-model="imageParams.content"
          type="text"
          placeholder="Content"
          class="flex-1 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />
        <input v-if="drawEnabled"
          v-model="imageParams.color"
          type="text"
          placeholder="Color"
          class="w-24 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />
        <input v-if="drawEnabled"
          v-model="imageParams.params"
          type="text"
          placeholder="Params"
          class="w-32 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />
        <input v-if="drawEnabled"
          v-model="imageParams.cords"
          type="text"
          placeholder="Cords"
          class="w-32 bg-gray-900 border border-gray-700 rounded px-2 py-1 text-sm"
        />

        <!-- Buttons -->
        <button
          class="px-3 py-2 bg-green-600 hover:bg-green-500 rounded"
          @click="generateImage"
        >
          Generate
        </button>

        <button
          v-if="drawEnabled"
          class="px-3 py-2 bg-purple-600 hover:bg-purple-500 rounded"
          @click="drawImage"
        >
          Draw
        </button>
      </template>
    </div>

  </div>

  <!-- Custom message box -->
  <div v-if="showMessageBox" class="message-box">
    <p>Do you want to apply AI changes?</p>
    <button @click="acceptChanges">Accept</button>
    <button @click="redoChanges">Redo</button>
  </div>
</template>

<script setup>
import { reactive, ref } from 'vue'
import { loginStatus, chat, messages, endOfMessages, generate, askAiDraw, updateImage } from './main.js'
import HexGrid from './HexGrid.vue';

const imageParams = reactive({
  cols: "",
  rows: "",
  content: "",
  color: "",
  params: "",
  cords: ""
})

const drawEnabled = ref(false);
const selectedMsg = ref(-1);
const cropedValue = ref('');

const newMessage = ref('')
const models = ref(['Llama3_1_8B', 'Qwen3_32B', 'Llama4Scout', 'Llama3_1_8B_Image', 'Llama4Scout_Image'])
const selectedModel = ref('Llama3_1_8B')

const minX = ref(0);
const minY = ref(0);

let tempBackup = '';

const showMessageBox = ref(false);

async function drawImage() {
  if (!imageParams.content.trim() || !imageParams.color.trim() || !imageParams.cords.trim()) return;
  const temp = "Draw, Connect and Fill with color: " + imageParams.color + "], with content: " + imageParams.content + ", with params: " + imageParams.params + ", at cords: " + imageParams.cords;
  try {
    tempBackup = messages.value[selectedMsg.value]?.content || '';
    const edited_pixels = await askAiDraw(temp, selectedModel.value, cropedValue.value);
    applyEditedPixels(edited_pixels);
    showMessageBox.value = true;
    //const edit = messages.value[selectedMsg.value].content;
    //await updateImage(loginStatus.principal, currentChatId.value, selectedMsg.value, edit);
  } catch (error) {
    alert(error);
  }
  
  imageParams.content = '';
  imageParams.color = '';
  imageParams.params = '';
  imageParams.cords = '';
}

const select_image = async (index) => {
  selectedMsg.value = index;
  drawEnabled.value = true;
};

const handleSelectCell = ({ x, y }) => {
  const msg = `(x:${x-minX.value} layer, y:${y-minY.value} layer)`;
  imageParams.cords += msg;
};

const handleSelectRect = (rect) => {
  if (!rect) {
    console.warn('rect is undefined!');
    return;
  }
  if (selectedMsg.value < 0) return;
  const msg = messages.value[selectedMsg.value];
  if (!msg || !msg.content) return;
  const content = msg.content;
  const { x1, y1, x2, y2 } = rect;
  cropedValue.value = cropStringImage(content, { x1, y1, x2, y2 });
};

function hasHex(content) {
  // sprawdzamy czy w tekście jest co najmniej jeden #RRGGBB
  const hexRegex = /#[0-9A-Fa-f]{6}/g;
  return hexRegex.test(content);
}

function cropStringImage(imageStr, rect) {
  // Wyciągamy nazwę
  const nameMatch = imageStr.match(/Content:\s*(.*)\nImage:/);
  const name = nameMatch ? nameMatch[1].trim() : "Unnamed";

  // Wyciągamy tylko część z obrazem
  const imageMatch = imageStr.match(/Image:\s*([\s\S]*)$/);
  const imageData = imageMatch ? imageMatch[1] : "";

  const regex = /\|y:(\d+),x:(\d+);(#?[0-9A-Fa-f]{6})\|/g;
  let match;
  const result = [];

  minX.value = rect.x2;
  minY.value = rect.y2;

  // nagłówek
  result.push(`Content: ${name}\nImage:\n`);

  while ((match = regex.exec(imageData)) !== null) {
    const y = parseInt(match[1]);
    const x = parseInt(match[2]);
    const color = match[3];

    if (x >= rect.x2 && x <= rect.x1 && y >= rect.y2 && y <= rect.y1) {
      const shiftedX = x - minX.value;
      const shiftedY = y - minY.value;
      result.push(`|y:${shiftedY},x:${shiftedX};${color}|`);
    }
  }

  return result.join('');
}

// Methods
async function generateImage() {
  if (!imageParams.content.trim()) return
  await generate(imageParams.content, imageParams.cols, imageParams.rows)
  imageParams.content = ''
  imageParams.cols = ''
  imageParams.rows = ''
}

async function sendMessage() {
  if (!newMessage.value.trim()) return

  await chat(newMessage.value, selectedModel.value)

  newMessage.value = ''
}

function micInput() {
  alert('Mic input not implemented')
}

const formatMarkdown = (text) => {
  let formatted = text;
  formatted = formatted.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
  formatted = formatted.replace(/^### (.*$)/gim, '<h3>$1</h3>');
  formatted = formatted.replace(/^## (.*$)/gim, '<h2>$1</h2>');
  formatted = formatted.replace(/^# (.*$)/gim, '<h1>$1</h1>');
  formatted = formatted.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  formatted = formatted.replace(/\*(.*?)\*/g, '<em>$1</em>');
  formatted = formatted.replace(/^\d+\.\s+(.*)/gm, '<li>$1</li>');
  formatted = formatted.replace(/(<li>.*<\/li>)/gs, '<ol>$1</ol>');
  formatted = formatted.replace(/^- (.*)/gm, '<li>$1</li>');
  formatted = formatted.replace(/(<li>.*<\/li>)/gs, '<ul>$1</ul>');
  formatted = formatted.replace(/\n{2,}/g, '</p><p>');
  formatted = `<p>${formatted}</p>`;

  return formatted;
}

function applyEditedPixels(edited_pixels) {
  if (selectedMsg.value < 0) return; // brak wybranego message
  
  // znajdź content obrazka w messages
  const msg = messages.value[selectedMsg.value];
  if (!msg || !msg.content) return;

  let content = msg.content;

  // dopasuj każdy pixel w formacie |y:1,x:1;#FF9900|
  const regex = /\|y:(\d+),x:(\d+);(#?[0-9A-Fa-f]{6})\|/g;
  let match;
  while ((match = regex.exec(edited_pixels)) !== null) {
    const y = parseInt(match[1]) + minY.value;
    const x = parseInt(match[2]) + minX.value;
    const color = match[3];

    // regex do znalezienia danego pixela w oryginalnym message
    const pixelRegex = new RegExp(`\\|y:${y},x:${x};#?[0-9A-Fa-f]{6}\\|`, "g");

    // zamiana starego koloru na nowy
    content = content.replace(pixelRegex, `|y:${y},x:${x};${color}|`);
  }

  // podmień content w messages
  messages.value[selectedMsg.value].content = content;
}

const acceptChanges = async () => {
  // Pobieramy nowy stan obrazu
  const edit = messages.value[selectedMsg.value].content;

  // Zapis na backend
  await updateImage(
    selectedMsg.value,
    edit
  );

  showMessageBox.value = false;
};

const redoChanges = () => {
  // Przywracamy kopię starego obrazu
  if (selectedMsg.value >= 0 && tempBackup) {
    messages.value[selectedMsg.value].content = tempBackup;
    //applyEditedPixels(tempBackup); // odśwież frontend
  }

  showMessageBox.value = false;
};
</script>

<style scoped>
/* optional: styling for scrollbar */
.flex-1::-webkit-scrollbar {
  width: 6px;
}
.flex-1::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}
.flex-1::-webkit-scrollbar-thumb:hover {
  background: #777;
}

.message {
  display: flex;
  max-width: 75%;
  padding: 0.8rem 1rem;
  border-radius: 14px;
  word-break: break-word;
  line-height: 1.5;
  box-shadow: 0 2px 6px rgba(0,0,0,0.05);
  animation: fadeIn 0.25s ease;
}
.ai-message {
  background: #e0f2fe;
  align-self: flex-start;
}
.user-message {
  background: #d1f7c4;
  align-self: flex-end;
}
.message-box {
  position: absolute;
  top: 20%;
  left: 50%;
  transform: translateX(-50%);
  background: white;
  border: 1px solid black;
  padding: 16px;
  z-index: 1000;
}
.message-block {
  margin-bottom: 1rem;
  text-align: left;
}
.message-date {
  text-align: center;
  font-size: 0.8rem;
  font-weight: 600;
  color: #666;
  margin: 0.5rem 0;
}
.message-wrapper {
  display: flex;
  flex-direction: column;
  max-width: 75%;
}
.user-wrapper { align-items: flex-end; margin-left: auto; }
.ai-wrapper { align-items: flex-start; margin-right: auto; }

.message-author {
  font-size: 0.8rem;
  font-weight: 600;
  margin-bottom: 0.2rem;
  color: #2563eb;
}
.user-wrapper .message-author { color: #16a34a; }

.message {
  padding: 0.8rem 1rem;
  border-radius: 12px;
  box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}
.user-message {
  background: rgba(209, 247, 196, 0.05); /* 75% opaque */
}
.ai-message {
  background: rgba(224, 242, 254, 0.05); /* 75% opaque */
}
.message-time {
  font-size: 0.7rem;
  color: #777;
  margin-top: 0.3rem;
}
</style>
