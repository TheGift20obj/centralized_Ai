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
        <button @click="openArchives">Archives</button>

        <!-- Okienko z archiwami -->
        <div v-if="showArchives" class="archive-modal">
          <h2 class="chat-name">Archived Chats</h2>
          <ul>
            <li v-for="chat in archives" :key="chat.id" class="archive-item">
              <span class="chat-name">{{ chat.name }}</span>
              <button @click="archiveChatAction(chat.id, false)">Restore</button>
            </li>
          </ul>
          <button class="close-btn" @click="showArchives = false">Close</button>
        </div>
        <ul>
          <li
            v-for="chat in chatList"
            :key="chat.id"
            :class="{ active: chat.id === currentChatId }"
          >
            <div style="display: flex; justify-content: space-between; width: 100%; align-items: center">
              <span @click="openChat(chat.id, chat.msg_len)">{{ chat.name }}</span>
              <div style="position: relative;">
                <button @click="showMenu = showMenu === chat.id ? null : chat.id">⋮</button>
                <div
                  v-if="showMenu === chat.id"
                  style="position: absolute; right: 0; background: white; color: black; border: 1px solid #ccc; padding: 4px; border-radius: 4px; z-index: 1;"
                >
                  <button @click="() => renameChatPrompt(chat.id, chat.name)">✏️ Rename</button><br />
                  <button @click="() => archiveChatAction(chat.id, true)">📦 Archive</button><br />
                  <button @click="() => removeChat(chat.id)">🗑️ Delete</button>
                </div>
              </div>
            </div>
          </li>
        </ul>
      </div>
    </div>

    <!-- Chat area -->
    <main class="chat-container">
      <section class="messages-container">
        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['message', msg.role === 'user' ? 'user-message' : 'ai-message']"
        >
          <template v-if="msg.role === 'user'">
            <span class="message-content">{{ msg.content }}</span>
            <span class="message-role_user">:{{ loginStatus.username }}<br>{{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { year: 'numeric', 
              month: '2-digit', 
              day: '2-digit', 
              hour: '2-digit', 
              minute: '2-digit'  }) }}</span>
          </template>
          <template v-else>
            <span class="message-role_ai">{{ msg.role }}:<br>{{ new Date(Number(msg.etc[0])).toLocaleTimeString([], { year: 'numeric', 
              month: '2-digit', 
              day: '2-digit', 
              hour: '2-digit', 
              minute: '2-digit'  }) }}</span>
            <template v-if="hasHex(msg.content)">
              <HexGrid 
                :content="msg.content" 
                :grid-cols="msg.etc[1]" 
                :grid-rows="msg.etc[2]"
                class="hex-grid" 
              />
            </template>

            <!-- jeżeli nie zawiera #HEX -->
            <template v-else>
              <span class="message-content" v-html="formatMarkdown(msg.content)"></span>
            </template>
          </template>
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
          {{ s }}
        </button>
      </section>

      <!-- Input and buttons -->
      <section class="input-container">
      <input
        v-model="userInput"
        @keyup.enter="sendMessage"
        :placeholder="aiWriting ? 'AI is writing...' : 'Type your message...'"
        class="chat-input"
        :disabled="aiWriting"
      />
      <button
        @click="sendMessage"
        class="btn-send"
        :disabled="!currentChatId || aiWriting"
      >
        Send
      </button>
      <button v-if="!isLoggedIn" @click="login" class="btn-login">Login</button>
      <span v-if="isLoggedIn" class="logged-in-text">
        ✅ Logged In as <strong>{{ loginStatus.username }}</strong>
        <button @click="showUsernameModal = true" class="btn-edit-username">✏️ Change Name</button>
        <label for="modelSelect"> Select Model: </label>
        <select id="modelSelect" v-model="selectedModel">
          <option value="Llama3_1_8B">Llama3_1_8B</option>
          <option value="Qwen3_32B">Qwen3_32B</option>
          <option value="Llama4Scout">Llama4Scout</option>
          <option value="Llama4Scout_Image">Llama4Scout_Image</option>
          <option value="Llama3_1_8B_Image">Llama3_1_8B_Image</option>
        </select>
        <!-- Dostosowanie rozmiaru tylko dla Llama4Scout_Image -->
        <div v-if="selectedModel === 'Llama4Scout_Image' || selectedModel === 'Llama3_1_8B_Image'" class="size-settings">
          <label>
            X:
            <input type="number" v-model.number="gridX" :min="8" :max="16"/>
          </label>
          <label>
            Y:
            <input type="number" v-model.number="gridY" :min="8" :max="16"/>
          </label>
        </div>
      </span>
    </section>
    </main>

    <!-- Modal for username change -->
    <div v-if="showUsernameModal" class="modal-overlay">
      <div class="modal-content">
        <h2>Change Username</h2>
        <input v-model="tempUsername" placeholder="Enter new name..." />
        <div class="modal-buttons">
          <button
            @click="() => { loginStatus.username = tempUsername; showUsernameModal = false; setUserName(loginStatus.principal, loginStatus.username); }"
          >Save</button>
          <button @click="() => { showUsernameModal = false }">Cancel</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
body, html {
  margin: 0;
  padding: 0;
  font-family: 'Inter', sans-serif;
  background-color: #f5f7fa;
  color: #333;
}

.app-wrapper {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ================== Sidebar ================== */
.sidebar {
  width: 50px;
  background-color: #1f2937;
  color: #fff;
  transition: width 0.3s;
  position: relative;
  display: flex;
  flex-direction: column;
}

.sidebar.open {
  width: 220px;
}

.sidebar-toggle {
  position: absolute;
  right: -15px;
  top: 20px;
  width: 30px;
  height: 30px;
  background-color: #1f2937;
  color: white;
  text-align: center;
  cursor: pointer;
  font-size: 18px;
  border-radius: 0 5px 5px 0;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 5px rgba(0,0,0,0.3);
}

.sidebar-content {
  padding: 15px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.sidebar-content h3 {
  margin-top: 0;
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 10px;
}

.sidebar-content button {
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 6px 10px;
  margin-bottom: 5px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.2s;
}

.sidebar-content button:hover {
  background: #2563eb;
}

.sidebar-content ul {
  list-style: none;
  padding: 0;
  margin-top: 10px;
  overflow-y: auto;
  flex: 1;
}

.sidebar-content li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 6px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.sidebar-content li:hover {
  background-color: rgba(255,255,255,0.1);
}

.sidebar-content li.active {
  background-color: #2563eb;
  font-weight: bold;
}

/* ================== Chat Area ================== */
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 1rem;
  background: #f5f7fa;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  padding: 1rem;
  background: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.05);
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.message {
  display: flex;
  align-items: flex-start;
  max-width: 70%;
  padding: 0.6rem 0.8rem;
  border-radius: 12px;
  gap: 0.5rem;
  word-break: break-word;
  line-height: 1.4;
}

.ai-message {
  background-color: #e0f2fe;
  align-self: flex-start;
}

.user-message {
  background-color: #d1f7c4;
  align-self: flex-end;
}

.message-role_ai {
  font-weight: 600;
  color: #555;
  align-self: flex-start;
  white-space: nowrap;
  text-align: right;
  flex-shrink: 0;
}

.message-role_user {
  font-weight: 600;
  color: #555;
  align-self: flex-end;
  white-space: nowrap;
  text-align: left;
  flex-shrink: 0;
}

.message-content pre {
  background: #f6f8fa;
  padding: 0.75rem;
  border-radius: 8px;
  overflow-x: auto;
  font-family: 'Fira Code', monospace;
}

.message-content h1, h2, h3 {
  margin: 0.5em 0 0.3em;
  font-weight: 700;
}

.message-content p {
  margin: 0.3em 0;
}

/* ================== Input ================== */
.input-container {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.chat-input {
  flex: 1;
  padding: 0.7rem 1rem;
  border-radius: 12px;
  border: 1px solid #ccc;
  font-size: 1rem;
  outline: none;
  transition: border 0.2s, box-shadow 0.2s;
}

.chat-input:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 5px rgba(59,130,246,0.3);
}

.btn-send, .btn-login {
  padding: 0.7rem 1.2rem;
  border-radius: 12px;
  border: none;
  background-color: #3b82f6;
  color: white;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}

.btn-send:disabled {
  background-color: #a5b4fc;
  cursor: not-allowed;
}

.btn-send:hover:not(:disabled),
.btn-login:hover {
  background-color: #2563eb;
}

/* ================== Suggestions ================== */
.suggestions-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.suggestion-btn {
  padding: 0.5rem 1rem;
  border-radius: 10px;
  border: 1px solid #ccc;
  background-color: #f0f0f0;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.2s, transform 0.1s;
}

.suggestion-btn:hover {
  background-color: #e0e0e0;
  transform: translateY(-1px);
}

/* ================== Modal ================== */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.modal-content {
  background: white;
  padding: 2rem;
  border-radius: 12px;
  width: 90%;
  max-width: 400px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.2);
}

.modal-buttons {
  margin-top: 1rem;
  display: flex;
  justify-content: space-between;
}

.btn-edit-username {
  margin-left: 0.5rem;
  padding: 0.3rem 0.6rem;
  font-size: 0.85rem;
}

/* ================== Hex Grid ================== */
.hex-grid {
  margin: 10px auto;
}
</style>
