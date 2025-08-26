<script setup>
import { ref, watch, onMounted, nextTick, computed } from 'vue';
import {
  archives, archive_chat,
  load,
  load_archives,
  chats, open, remove_chat, rename_chat, current,
  load_images, images
} from './main.js';
import Sidebar from './Sidebar.vue';
import Chat from './Chat.vue';
import HexGrid from './HexGrid.vue';

const showMenu = ref(null);

const filteredChats = computed(() => {
  if (!searchQuery.value) return chats.value;
  return chats.value.filter(chat =>
    chat.name.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

async function renameChatPrompt(id, oldName) {
  const newName = prompt("Enter new chat name:", oldName)
  if (newName && newName !== oldName) {
    await rename_chat(id, newName)
  }
  showMenu.value = null
}

async function archiveChatAction(id, archive) {
  await archive_chat(id, archive)
  showMenu.value = null
}

async function removeChat(id) {
  if (confirm("Are you sure you want to delete this chat? This action cannot be undone.")) {
    await remove_chat(id)
  }
  showMenu.value = null
}

/*
const ctxName = ref('Moja Grafika');
const cols = ref(3);
const rows = ref(3);
const ctxName = ref('');

function demoColorGetter(y, x) {
  //if (y === x) return '#FF9900';
  return '#E5E7EB'; // jasna szarość
}

function generateImageDescriptor(name, cols, rows, getColorAt) {
  // helper losujący powtarzalny kolor dla (y,x), gdy nie podano getColorAt
  const fallbackColor = (y, x) => {
    // prosty determinizm: hasz z y,x
    const seed = (y * 73856093) ^ (x * 19349663);
    // trzy kanały z seed
    const r = (seed & 0xff);
    const g = ((seed >> 8) & 0xff);
    const b = ((seed >> 16) & 0xff);
    const toHex = (n) => n.toString(16).padStart(2, '0').toUpperCase();
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  };

  const colorAt = typeof getColorAt === 'function' ? getColorAt : fallbackColor;

  const lines = [];
  lines.push(`Content: ${name}`);
  lines.push(`Image:`);
  for (let y = 1; y <= rows; y++) {
    const rowParts = [];
    for (let x = 1; x <= cols; x++) {
      rowParts.push(`|y:${y},x:${x};${colorAt(y, x)}|`);
    }
    lines.push(rowParts.join(''));
  }
  return lines.join('');
}

const selectedModel = ref("Llama3_1_8B");

const gridX = ref(16);
const gridY = ref(16);

function hasHex(content) {
  // sprawdzamy czy w tekście jest co najmniej jeden #RRGGBB
  const hexRegex = /#[0-9A-Fa-f]{6}/g;
  return hexRegex.test(content);
}*/

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
/*
const messages = ref([]);
const userInput = ref('');
const isLoggedIn = ref(false);
const showSidebar = ref(false);
const aiWriting = ref(false);
const selectedMsg = ref(-1);
const endOfMessages = ref(null);

const chatList = ref([]);
const currentChatId = ref(null);
const currentChatName = ref('');
const minX = ref(0);
const minY = ref(0);

const showUsernameModal = ref(false);
const tempUsername = ref('');
const showMenu = ref(null);

const suggestions = ref([]);

const showArchives = ref(false);
const archives = ref([]);
const cropedValue = ref('');

const showMessageBox = ref(false);
let tempBackup = ''; // Kopia starego obrazka
/*
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

const handleSelectCell = ({ x, y }) => {
  const msg = `(x:${x-minX.value} layer, y:${y-minY.value} layer)`;
  userInput.value += " " + msg;
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

const onGenerate = async () => {
  const userMsg = { role: 'user', content: generateImageDescriptor(ctxName.value, cols.value, rows.value, demoColorGetter), etc: [Date.now(), cols.value, rows.value] };
  const temp = userMsg.content;
  messages.value.push(userMsg);
  await addChatMessage(
    loginStatus.principal,
    currentChatId.value,
    temp,
    'user', cols.value, rows.value, Date.now()
  );
}

const loadSuggestions = () => {
  suggestions.value = getRandomUserMessages();
};

const archiveChatAction = async (chatId, archive) => {
  await archiveChat(loginStatus.principal, chatId, archive);
  if (showArchives.value) {
    await openArchives();
  }
  if (archive && currentChatId.value === chatId) {
    currentChatId.value = null;
    messages.value = [];
    currentChatName.value = '';
    selectedMsg.value = -1;
  }
  await loadChats();
};

const loadChats = async () => {
  if (!loginStatus.loggedIn) return;
  chatList.value = await listChats(loginStatus.principal, false);
};

const openArchives = async () => {
  if (!loginStatus.loggedIn) return;
  showArchives.value = true;
  archives.value = await listChats(loginStatus.principal, true);
};

const openChat = async (chatId, msgLen) => {
  if (!loginStatus.loggedIn) return;
  selectedMsg.value = -1;
  currentChatId.value = chatId;
  //currentChatName.value = name;
  const result = await getChatHistory(loginStatus.principal, chatId, msgLen);
  if (!result || !result.messages) {
    messages.value = [];
    return;
  }
  messages.value = result.messages.flatMap(m => [
    { role: m.role, content: m.content, etc: m.etc },
  ]);
  //currentChatName.value = result.name;
  nextTick(scrollToBottom);
  loadSuggestions();
};

const createChat = async () => {
  if (!loginStatus.loggedIn) return;
  const uuid = crypto.randomUUID();
  const bytes = Uint8Array.from(
    uuid.replace(/-/g, '').match(/.{2}/g).map(b => parseInt(b, 16))
  );
  const name = `New Chat ${chatList.value.length + 1}`;
  await createNewChat(loginStatus.principal, bytes, name);
  await loadChats();
  await openChat(bytes, 0);
};

const askAiDrawVue = async () => {
  aiWriting.value = true;
  const temp = userInput.value;
  userInput.value = '';
  try {
    tempBackup = messages.value[selectedMsg.value]?.content || '';
    const edited_pixels = await askAiDraw(temp, selectedModel.value, cropedValue.value);
    applyEditedPixels(edited_pixels);
    showMessageBox.value = true;
    //const edit = messages.value[selectedMsg.value].content;
    //await updateImage(loginStatus.principal, currentChatId.value, selectedMsg.value, edit);
    aiWriting.value = false;
  } catch (error) {
    alert(error);
    aiWriting.value = false;
  }
}

const acceptChanges = async () => {
  // Pobieramy nowy stan obrazu
  const edit = messages.value[selectedMsg.value].content;

  // Zapis na backend
  await updateImage(
    loginStatus.principal,
    currentChatId.value,
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

const sendMessage = async () => {
  if (!userInput.value.trim() || !currentChatId.value) return;
  const canDo = await tryPrompt(loginStatus.principal);
  if (!canDo) {
    alert("Daily limit reached. Come back tomorrow!");
    return;
  }

  aiWriting.value = true;

  const userMsg = { role: 'user', content: userInput.value, etc: [Date.now(), 0, 0] };
  messages.value.push(userMsg);

  const temp = userInput.value;
  userInput.value = '';
  await addChatMessage(
    loginStatus.principal,
    currentChatId.value,
    temp,
    'user', 0, 0, Date.now()
  );

  try {
    //aiWriting.value = true;
    const reply = await chatWithBackend(temp, gridX.value, gridY.value, selectedModel.value, loginStatus.principal, currentChatId.value, messages.value.length);
    messages.value.push({ role: selectedModel.value, content: reply, etc: [Date.now(), gridX.value, gridY.value] });
    await addChatMessage(
      loginStatus.principal,
      currentChatId.value,
      reply,
      selectedModel.value, gridX.value, gridY.value, Date.now()
    );
    aiWriting.value = false;
  } catch (error) {
    messages.value.push({ role: selectedModel.value, content: "Sorry, I'm currently overloaded. Could you please try again?", etc: [Date.now(), gridX.value, gridY.value] });
    await addChatMessage(
      loginStatus.principal,
      currentChatId.value,
      "Sorry, I'm currently overloaded. Could you please try again?",
      selectedModel.value, gridX.value, gridY.value
    );
    aiWriting.value = false;
  }

  await nextTick();
  scrollToBottom();
  loadSuggestions();
};

const select_image = async (index) => {
  selectedMsg.value = index;
};

const sendSuggestion = async (msg) => {
  userInput.value = msg;
  //await sendMessage();
};

const scrollToBottom = () => {
  endOfMessages.value?.scrollIntoView({ behavior: 'smooth' });
};

const removeChat = async (chatId) => {
  await deleteChat(loginStatus.principal, chatId);
  await loadChats();
  if (chatId === currentChatId.value) {
    messages.value = [];
    currentChatId.value = null;
    currentChatName.value = '';
    selectedMsg.value = -1;
  }
};

const renameChatPrompt = async (chatId, oldName) => {
  const newName = prompt('Rename chat:', oldName);
  if (newName && newName.trim()) {
    await renameChat(loginStatus.principal, chatId, newName);
    await loadChats();
  }
};

onMounted(async () => {
  const check = setInterval(async () => {
    if (loginStatus.loggedIn) {
      isLoggedIn.value = true;
      clearInterval(check);
      await loadChats();
      loadSuggestions();
    }
  }, 300);
});
*/
const showSearch = ref(false)
const showLibrary = ref(false)
const showArchives = ref(false)

const searchQuery = ref('');

async function openChat(id) {
  await open(id)
}

function search() {
  showSearch.value = true
  showLibrary.value = false
  showArchives.value = false
}
async function openLibrary() {
  await load_images()
  showLibrary.value = true
  showSearch.value = false
  showArchives.value = false
}
async function openArchives() {
  await load_archives()
  showArchives.value = true
  showLibrary.value = false
  showSearch.value = false
}
function closeAll() {
  showSearch.value = false
  showLibrary.value = false
  showArchives.value = false
  searchQuery.value = '';
}

const gridCols = computed(() => {
  if (images.value.length <= 2) return images.value.length;
  if (images.value.length <= 4) return 2;
  if (images.value.length <= 9) return 3;
  if (images.value.length <= 16) return 4;
  return 5; // dla dużej liczby
});

function hasHex(content) {
  // sprawdzamy czy w tekście jest co najmniej jeden #RRGGBB
  const hexRegex = /#[0-9A-Fa-f]{6}/g;
  return hexRegex.test(content);
}


async function rearchiveChatAction(id, archive) {
  await archive_chat(id, archive)
  await load_archives()
  await load()
}
</script>

<template>
  <div class="flex h-screen w-screen">
    <!-- Sidebar -->
    <aside>
      <Sidebar 
        @search="search"
        @open-library="openLibrary"
        @open-archives="openArchives"
      />
    </aside>

    <!-- Modal Search -->
    <div
      v-if="showSearch"
      class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50"
    >
      <div class="bg-gray-900 text-gray-100 rounded-xl shadow-2xl w-1/2 h-3/4 p-6 relative">
        <h2 class="text-lg font-semibold mb-4">Search</h2>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Type to search..."
          class="w-full bg-gray-800 border border-gray-700 p-2 rounded mb-4 text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          class="absolute top-2 right-2 text-gray-400 hover:text-gray-200"
          @click="closeAll"
        >
          ✖
        </button>

        <!-- Lista dopasowanych chatów -->
        <ul class="space-y-1">
          <li
            v-for="chat in filteredChats"
            :key="chat.id"
            :class="[
              'p-2 rounded cursor-pointer hover:bg-gray-800 flex justify-between items-center',
              chat.id === current ? 'bg-gray-700 text-white' : 'text-gray-200'
            ]"
          >
            <span 
              class="truncate flex-1" 
              @click="openChat(chat.id)"
            >
            {{ chat.name }}
            </span>
            <!-- Actions -->
            <div class="relative">
              <button 
                class="px-2 py-1 rounded hover:bg-gray-700" 
                @click="showMenu = showMenu === chat.id ? null : chat.id"
              >
              ⋮
              </button>

              <!-- Dropdown -->
              <div 
                  v-if="showMenu === chat.id" 
                  class="absolute right-0 mt-1 w-40 bg-gray-800 rounded shadow-lg z-10"
              >
                <button 
                  @click="renameChatPrompt(chat.id, chat.name)" 
                  class="block w-full text-left px-3 py-2 hover:bg-gray-700"
                >
                ✏️&nbsp;Rename
                </button>
                <button 
                  @click="archiveChatAction(chat.id, true)" 
                  class="block w-full text-left px-3 py-2 hover:bg-gray-700"
                >
                📦&nbsp;Archive
                </button>
                <button 
                  @click="removeChat(chat.id)" 
                  class="block w-full text-left px-3 py-2 text-red-400 hover:bg-gray-700"
                >
                🗑️&nbsp;Delete
                </button>
              </div>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <!-- Modal Library -->
    <div
      v-if="showLibrary"
      class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50"
    >
      <div class="bg-gray-900 text-gray-100 rounded-xl shadow-2xl w-5/6 h-5/6 p-6 relative overflow-y-auto">
        <h2 class="text-lg font-semibold mb-4">Library</h2>

        <!-- GRID -->
        <div
          v-if="images.length"
          class="grid gap-4"
          :style="{
            gridTemplateColumns: 'repeat(' + gridCols + ', minmax(0, 1fr))',
            gridAutoRows: 'minmax(150px, auto)'
          }"
        >
          <div
            v-for="(msg, index) in images"
            :key="index"
            class="bg-gray-800 rounded-xl p-2 flex items-center justify-center"
          >
            <!-- użycie HexGrid -->
            <HexGrid
              v-if="hasHex(msg.image)"
              :content="msg.image"
              :grid-cols="msg.etc[1]"
              :grid-rows="msg.etc[2]"
              :style="{ aspectRatio: msg.etc[1] + ' / ' + msg.etc[2], width: '100%' }"
            />
          </div>
        </div>

        <p v-else class="text-gray-400">No images found.</p>

        <button
          class="absolute top-2 right-2 text-gray-400 hover:text-gray-200"
          @click="closeAll"
        >
          ✖
        </button>
      </div>
    </div>

    <!-- Modal Archives -->
    <div
      v-if="showArchives"
      class="fixed inset-0 bg-black bg-opacity-70 flex items-center justify-center z-50"
    >
      <div class="bg-gray-900 text-gray-100 rounded-xl shadow-2xl w-1/2 h-3/4 p-6 relative flex flex-col">
        <h2 class="text-lg font-semibold mb-4">Archives</h2>
        <p class="text-gray-300">Your archived chats and files will appear here.</p>
        <section class="flex-1 min-h-0 overflow-y-auto pr-1">
          <ul class="space-y-1">
            <li
              v-for="chat in archives"
              :key="chat.id"
              class="p-2 rounded cursor-pointer hover:bg-gray-800 flex justify-between items-center"
            >
              <span class="truncate flex-1">{{ chat.name }}</span>
              <button
                @click="rearchiveChatAction(chat.id, false)"
                class="px-2 py-1 rounded hover:bg-gray-700"
              >
                Restore
              </button>
            </li>
          </ul>
        </section>
        <button
          class="absolute top-2 right-2 text-gray-400 hover:text-gray-200"
          @click="closeAll"
        >
          ✖
        </button>
      </div>
    </div>


    <!-- Main content area -->
    <main class="w-full">
      <Chat />
    </main>
  </div>
</template>

<style scoped>
/*
<template>
  <div class="app-wrapper">
    <!-- Sidebar -->
    <div class="sidebar" :class="{ open: showSidebar }">
      <div class="sidebar-toggle" @click="showSidebar = !showSidebar">
        <span>{{ showSidebar ? '<' : '>' }}</span>
      </div>
      <div v-if="showSidebar" class="sidebar-content">
        <h3>Chat List</h3>
        <button @click="createChat">+ New Chat</button>
        <button @click="openArchives">📦 Archives</button>

        <!-- Archive Modal -->
        <div v-if="showArchives" class="modal-overlay">
          <div class="modal-content">
            <h3>Archived Chats</h3>
            <section class="chat-list-container">
              <ul class="archive-list">
                <li v-for="chat in archives" :key="chat.id" class="archive-item">
                  <span class="chat-name">{{ chat.name }}</span>
                  <button @click="archiveChatAction(chat.id, false)">Restore</button>
                </li>
              </ul>
            </section>
            <div class="modal-buttons">
              <button class="btn-secondary" @click="showArchives = false">Close</button>
            </div>
          </div>
        </div>

        <!-- Chat list -->
        <section class="chat-list-container">
          <ul class="chat-list">
            <li
              v-for="chat in chatList"
              :key="chat.id"
              :class="{ active: chat.id === currentChatId }"
              class="chat-item"
            >
              <div class="chat-row">
                <span class="chat-name" @click="openChat(chat.id, chat.msg_len)">
                  {{ chat.name }}
                </span>
                <div class="chat-actions">
                  <button class="dots-btn" @click="showMenu = showMenu === chat.id ? null : chat.id">⋮</button>
                  
                  <!-- Dropdown -->
                  <div v-if="showMenu === chat.id" class="chat-menu">
                    <button @click="() => renameChatPrompt(chat.id, chat.name)" class="cass">✏️ Rename</button>
                    <button @click="() => archiveChatAction(chat.id, true)" class="cass">📦 Archive</button>
                    <button @click="() => removeChat(chat.id)" class="danger">🗑️ Delete</button>
                  </div>
                </div>
              </div>
            </li>
          </ul>
        </section>
      </div>
    </div>

    <!-- Chat area -->
    <main class="chat-container">
      <!-- Messages -->
      <section class="messages-container">
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
            <div class="message user-message" :class="{ 'has-hex': hasHex(msg.content) }">
              <template v-if="hasHex(msg.content)">
                <HexGrid :content="msg.content" :grid-cols="msg.etc[1]" :grid-rows="msg.etc[2]" @selectCell="handleSelectCell" @selectArea="handleSelectRect" class="hex-grid" :style="{ aspectRatio: msg.etc[1] + ' / ' + msg.etc[2] }"/>
                <div v-if="selectedModel.includes('Image')">
                  <div v-if="selectedMsg === index">
                    <span>Selected</span>
                  </div>
                  <div v-else>
                    <button @click="select_image(index)">Select</button>
                  </div>
                </div>
              </template>
              <template v-else>
                <span class="message-content">{{ msg.content }}</span>
              </template>
            </div>
            <div class="message-time">
              {{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
            </div>
          </div>

          <!-- AI -->
          <div v-else class="message-wrapper ai-wrapper">
            <div class="message-author">AI</div>
            <div class="message ai-message">
              <template v-if="hasHex(msg.content)">
                <HexGrid :content="msg.content" :grid-cols="msg.etc[1]" :grid-rows="msg.etc[2]" @selectCell="handleSelectCell" class="hex-grid" :style="{ aspectRatio: msg.etc[1] + ' / ' + msg.etc[2] }"/>
              </template>
              <template v-else>
                <span class="message-content" v-html="formatMarkdown(msg.content)"></span>
              </template>
            </div>
            <div class="message-time">
              {{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}
            </div>
          </div>
        </div>
        <div ref="endOfMessages" />
      </section>

      <!-- Suggestions -->
      <section v-if="suggestions.length && currentChatId" class="suggestions-container">
        <button
          v-for="(s, i) in suggestions"
          :key="i"
          class="suggestion-btn"
          @click="sendSuggestion(s)"
        >
          💡 {{ s }}
        </button>
      </section>

      <!-- Input -->
      <section class="input-container">
        <input
          v-model="userInput"
          @keyup.enter="sendMessage"
          :placeholder="aiWriting ? 'AI is writing...' : 'Type your message...'"
          class="chat-input"
          :disabled="aiWriting"
        />
        <button @click="sendMessage" class="btn-send" :disabled="!currentChatId || aiWriting || selectedModel.includes('Image')">
          Send
        </button>
      </section>

      <!-- Footer Controls -->
      <section class="footer-controls">
        <div v-if="!isLoggedIn">
          <button @click="login" class="btn-login">Login</button>
        </div>
        <div v-else class="user-controls">
          <span class="logged-in-badge">✅ {{ loginStatus.username }}</span>
          <button @click="showUsernameModal = true" class="btn-small">✏️ Change Name</button>
          
          <!-- Model selector -->
          <div class="model-settings">
            <label for="modelSelect">Model:</label>
            <select id="modelSelect" v-model="selectedModel">
              <option value="Llama3_1_8B">Llama3_1_8B</option>
              <option value="Qwen3_32B">Qwen3_32B</option>
              <option value="Llama4Scout">Llama4Scout</option>
              <option value="Llama4Scout_Image">Llama4Scout_Image</option>
              <option value="Llama3_1_8B_Image">Llama3_1_8B_Image</option>
            </select>
          </div>

          <!-- Grid size (only for image models) -->
          <div v-if="selectedModel.includes('Image')" class="size-settings">
            <label>X:<input type="number" v-model.number="gridX" :min="8" :max="16"/></label>
            <label>Y:<input type="number" v-model.number="gridY" :min="8" :max="16"/></label>

            <div style="display:flex; gap:8px; align-items:center; margin-bottom:8px;">
              <input v-model="ctxName" placeholder="Nazwa (Context)" />
              <input type="number" v-model.number="cols" min="1" placeholder="x (cols)" />
              <input type="number" v-model.number="rows" min="1" placeholder="y (rows)" />
              <button @click="onGenerate">Generate Empty Image</button>
              <button v-if="selectedMsg >= 0" @click="askAiDrawVue" :disabled="aiWriting">Ask AI Draw</button>
            </div>
          </div>
        </div>
      </section>
    </main>

    <!-- Modal: Change Username -->
    <div v-if="showUsernameModal" class="modal-overlay">
      <div class="modal-content">
        <h2>Change Username</h2>
        <input v-model="tempUsername" placeholder="Enter new name..." />
        <div class="modal-buttons">
          <button class="btn-primary"
            @click="() => { loginStatus.username = tempUsername; showUsernameModal = false; setUserName(loginStatus.principal, loginStatus.username); }"
          >Save</button>
          <button class="btn-secondary" @click="() => { showUsernameModal = false }">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Custom message box -->
    <div v-if="showMessageBox" class="message-box">
      <p>Do you want to apply AI changes?</p>
      <button @click="acceptChanges">Accept</button>
      <button @click="redoChanges">Redo</button>
    </div>
  </div>
</template>
*/
</style>