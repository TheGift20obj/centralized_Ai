<script setup>
import { ref, watch, onMounted, nextTick } from 'vue';
import {
  login,
  loginStatus,
  chatWithBackend,
  createNewChat,
  addChatMessage,
  getChatHistory,
  listChats,
  deleteChat,
  renameChat,
  setUserName,
  getUserName,
  tryPrompt,
  getRandomUserMessages,
  archiveChat,
} from './main.js';
import HexGrid from './HexGrid.vue';

const selectedModel = ref("Llama3_1_8B");

const gridX = ref(16);
const gridY = ref(16);

function hasHex(content) {
  // sprawdzamy czy w tekście jest co najmniej jeden #RRGGBB
  const hexRegex = /#[0-9A-Fa-f]{6}/g;
  return hexRegex.test(content);
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
};

const messages = ref([]);
const userInput = ref('');
const isLoggedIn = ref(false);
const showSidebar = ref(false);
const aiWriting = ref(false);
const endOfMessages = ref(null);

const chatList = ref([]);
const currentChatId = ref(null);
const currentChatName = ref('');

const showUsernameModal = ref(false);
const tempUsername = ref('');
const showMenu = ref(null);

const suggestions = ref([]);

const showArchives = ref(false);
const archives = ref([]);

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
</script>

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
            <div class="message user-message">
              <span class="message-content">{{ msg.content }}</span>
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
                <HexGrid :content="msg.content" :grid-cols="msg.etc[1]" :grid-rows="msg.etc[2]" class="hex-grid"/>
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
        <button @click="sendMessage" class="btn-send" :disabled="!currentChatId || aiWriting">
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
  </div>
</template>


<style scoped>
/* ================== Global ================== */
body, html {
  margin: 0;
  padding: 0;
  font-family: 'Inter', sans-serif;
  background: linear-gradient(135deg, #eef2f3, #dfe9f3);
  color: #333;
  height: 100%;
  overflow: hidden;
}

.app-wrapper {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ================== Sidebar ================== */
.sidebar {
  width: 60px;
  background: rgba(31, 41, 55, 0.9);
  backdrop-filter: blur(8px);
  color: #fff;
  transition: width 0.35s ease;
  position: relative;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(255,255,255,0.1);
}

.sidebar.open {
  width: 250px;
}

.sidebar-toggle {
  position: absolute;
  right: -15px;
  top: 20px;
  width: 34px;
  height: 34px;
  background: #2563eb;
  color: white;
  text-align: center;
  cursor: pointer;
  font-size: 18px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0,0,0,0.25);
  transition: background 0.25s;
}
.sidebar-toggle:hover {
  background: #1d4ed8;
}

.sidebar-content {
  padding: 18px;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-content h3 {
  font-size: 1.2rem;
  font-weight: 700;
  margin-bottom: 12px;
}

.sidebar-content button {
  background: #2563eb;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 7px 12px;
  margin-bottom: 6px;
  cursor: pointer;
  font-size: 0.95rem;
  transition: all 0.25s;
}
.sidebar-content button:hover {
  background: #1d4ed8;
  transform: translateY(-1px);
}

.sidebar-content ul {
  list-style: none;
  padding: 0;
  margin-top: 12px;
  overflow-y: auto;
  flex: 1;
  scrollbar-width: thin;
}

.sidebar-content li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 9px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.25s;
}
.sidebar-content li:hover {
  background: rgba(255,255,255,0.15);
}
.sidebar-content li.active {
  background: #3b82f6;
  font-weight: bold;
}

/* ================== Chat Area ================== */
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
  background: #f9fafb;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  padding: 1.2rem;
  background: #ffffff;
  border-radius: 16px;
  box-shadow: 0 8px 18px rgba(0,0,0,0.06);
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  scrollbar-width: thin;
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

.message-role_ai,
.message-role_user {
  font-size: 0.75rem;
  font-weight: 600;
  color: #666;
  margin-top: 0.3rem;
  opacity: 0.85;
}

.message-content pre {
  background: #f6f8fa;
  padding: 0.75rem;
  border-radius: 10px;
  overflow-x: auto;
  font-family: 'Fira Code', monospace;
  font-size: 0.9rem;
}

/* ================== Input ================== */
.input-container {
  display: flex;
  gap: 0.6rem;
  margin-top: 1rem;
}

.chat-input {
  flex: 1;
  padding: 0.8rem 1rem;
  border-radius: 14px;
  border: 1px solid #ddd;
  font-size: 1rem;
  outline: none;
  transition: all 0.25s;
  background: #fff;
}
.chat-input:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 6px rgba(59,130,246,0.35);
}

.btn-send, .btn-login {
  padding: 0.8rem 1.2rem;
  border-radius: 14px;
  border: none;
  background: #2563eb;
  color: white;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.25s;
}
.btn-send:disabled {
  background: #93c5fd;
  cursor: not-allowed;
}
.btn-send:hover:not(:disabled),
.btn-login:hover {
  background: #1d4ed8;
  transform: translateY(-1px);
}

/* ================== Suggestions ================== */
.suggestions-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
  margin-bottom: 0.6rem;
}
.suggestion-btn {
  padding: 0.5rem 1rem;
  border-radius: 12px;
  border: 1px solid #ccc;
  background: #f9f9f9;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}
.suggestion-btn:hover {
  background: #f1f5f9;
  transform: translateY(-1px);
}

/* ================== Modal ================== */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(15, 23, 42, 0.5);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}
.modal-content {
  background: #d4e0e2;
  padding: 1rem;
  border-radius: 8px;
  width: 70%;
  height: 700px;
  max-width: 420px;
  box-shadow: 0 12px 36px rgba(0,0,0,0.25);
  animation: fadeIn 0.3s ease;
}
.modal-buttons {
  margin-top: 1.2rem;
  display: flex;
  justify-content: flex-end;
  gap: 0.8rem;
}
.btn-edit-username {
  margin-left: 0.6rem;
  padding: 0.35rem 0.7rem;
  font-size: 0.85rem;
}

/* ================== Hex Grid ================== */
.hex-grid {
  margin: 12px auto;
}

/* Message Layout */
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
  background: #d1f7c4;
}
.ai-message {
  background: #e0f2fe;
}
.message-time {
  font-size: 0.7rem;
  color: #777;
  margin-top: 0.3rem;
}

/* Archive fix */
.archive-item {
  background: #f9fafb;
  padding: 0.6rem 0.8rem;
  border-radius: 8px;
  margin-bottom: 0.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.archive-item .chat-name {
  color: #111;
  font-weight: 500;
}


.chat-item {
  display: flex;
  align-items: center;
  justify-content: space-between; /* Nazwa po lewej, przycisk po prawej */
  padding: 8px 12px;
  border-bottom: 1px solid #ddd;
}

.chat-item.active {
  background: #e5f3ff;
}

.chat-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.chat-name {
  cursor: pointer;
  flex-grow: 1;
}

.chat-actions {
  position: relative;
}

.dots-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
}

/* Dropdown styled like <select> options */
.chat-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: white;
  border: 1px solid #ccc;
  border-radius: 6px;
  box-shadow: 0 4px 8px rgba(0,0,0,0.12);
  z-index: 20;
  min-width: 140px;
  display: flex;
  flex-direction: column;
}

.chat-menu button {
  padding: 8px 12px;
  text-align: left;
  background: white;
  border: none;
  cursor: pointer;
  font-size: 0.9rem;
}

.chat-menu button:hover {
  background: #f0f0f0;
}

.chat-menu button.danger {
  color: #dc2626;
}

.chat-menu button.cass {
  color: #455355;
}

.chat-menu button.danger:hover {
  background: #fee2e2;
}

.chat-list-container {
  display: flex;
  max-height: 83.75%; /* Cały sidebar maksymalnie 80% wysokości ekranu */
  flex-grow: 1;
  overflow-y: auto;
  flex-direction: column;
  scrollbar-width: thin;
}
/* ================== Animations ================== */
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>