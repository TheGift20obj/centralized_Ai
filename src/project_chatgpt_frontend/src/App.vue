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
} from './main.js';

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

const loadSuggestions = () => {
  suggestions.value = getRandomUserMessages();
};

const loadChats = async () => {
  if (!loginStatus.loggedIn) return;
  const list = await listChats(loginStatus.principal);
  chatList.value = list;
};

const openChat = async (chatId, msgLen) => {
  currentChatId.value = chatId;
  //currentChatName.value = name;
  const result = await getChatHistory(loginStatus.principal, chatId, msgLen);
  if (!result || !result.messages) {
    messages.value = [];
    return;
  }
  messages.value = result.messages.flatMap(m => [
    { role: m.role, content: m.content, timestamp: m.timestamp },
  ]);
  //currentChatName.value = result.name;
  nextTick(scrollToBottom);
  loadSuggestions();
};

const createChat = async () => {
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

  const userMsg = { role: 'user', content: userInput.value, timestamp: Date.now() };
  messages.value.push(userMsg);

  const temp = userInput.value;
  userInput.value = '';
  await addChatMessage(
    loginStatus.principal,
    currentChatId.value,
    temp,
    'user'
  );

  try {
    //aiWriting.value = true;
    const reply = await chatWithBackend(temp);
    messages.value.push({ role: 'ai', content: reply, timestamp: Date.now() });
    await addChatMessage(
      loginStatus.principal,
      currentChatId.value,
      reply,
      'ai'
    );
    aiWriting.value = false;
  } catch (error) {
    messages.value.push({ role: 'ai', content: 'Connection error! ' + error.message, timestamp: Date.now() });
  }

  await nextTick();
  scrollToBottom();
  loadSuggestions();
};

const sendSuggestion = async (msg) => {
  userInput.value = msg;
  await sendMessage();
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
          :class="['message', msg.role === 'ai' ? 'ai-message' : 'user-message']"
        >
          <template v-if="msg.role === 'ai'">
            <span class="message-role">AI:</span>
            <span class="message-content" v-html="formatMarkdown(msg.content)"></span>
          </template>
          <template v-else>
            <span class="message-content">{{ msg.content }}</span>
            <span class="message-role">:{{ loginStatus.username }}</span>
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
  border-radius: 8px;
  min-width: 300px;
  max-width: 90%;
}

.modal-buttons {
  margin-top: 1rem;
  display: flex;
  justify-content: space-between;
}

.btn-edit-username {
  margin-left: 0.5rem;
  padding: 0.3rem 0.6rem;
  font-size: 0.8rem;
}

.app-wrapper {
  display: flex;
  height: 100vh;
  overflow: hidden;
  font-family: 'Segoe UI', sans-serif;
}

.sidebar {
  width: 40px;
  background-color: #1e1e2f;
  color: white;
  transition: width 0.3s;
  position: relative;
}

.sidebar.open {
  width: 240px;
}

.sidebar-toggle {
  position: absolute;
  right: -20px;
  top: 20px;
  width: 20px;
  height: 40px;
  background-color: #1e1e2f;
  color: white;
  text-align: center;
  cursor: pointer;
  font-size: 20px;
  border-radius: 0 5px 5px 0;
}

.sidebar-content {
  padding: 10px;
}

.sidebar-content ul {
  list-style: none;
  padding: 0;
}

.sidebar-content li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
  cursor: pointer;
  border-bottom: 1px solid #444;
}

.sidebar-content li.active {
  font-weight: bold;
  color: #00d1b2;
}

.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: #f0f2f5;
  padding: 1rem;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  padding: 1rem;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.05);
}

.message {
  display: flex;
  align-items: center;
  margin-bottom: 0.75rem;
  padding: 0.5rem;
  border-radius: 6px;
  max-width: 70%;
  gap: 0.5rem;
}

.ai-message {
  justify-content: flex-start;
  background: #e3f2fd;
  align-self: flex-start;
  text-align: left;
}

.user-message {
  justify-content: flex-end;
  background: #d1f7c4;
  align-self: flex-end;
  text-align: right;
}

.message-role {
  font-weight: bold;
  margin: 0 5px;
  color: #555;
  align-self: flex-start;
}

.input-container {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.suggestions-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.suggestion-btn {
  padding: 0.4rem 0.8rem;
  background-color: #f0f0f0;
  border: 1px solid #ccc;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.2s;
}

.suggestion-btn:hover {
  background-color: #e0e0e0;
}

.chat-input {
  flex: 1;
  padding: 0.6rem;
  border-radius: 6px;
  border: 1px solid #ccc;
  font-size: 1rem;
}

.btn-send, .btn-login {
  padding: 0.6rem 1.2rem;
  background-color: #00d1b2;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: bold;
}

.btn-send:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.logged-in-text {
  align-self: center;
  font-size: 0.9rem;
  color: green;
  font-weight: bold;
}
.message-content pre {
  background: #f6f8fa;
  padding: 0.75rem;
  border-radius: 6px;
  overflow-x: auto;
  font-family: monospace;
  white-space: pre-wrap;
}

.message-content h1, h2, h3 {
  margin: 0.5em 0 0.3em;
  font-weight: bold;
}

.message-content ul, .message-content ol {
  padding-left: 1.5em;
  margin: 0.3em 0;
}

.message-content li {
  margin-bottom: 0.3em;
}

.message-content strong {
  font-weight: bold;
}

.message-content em {
  font-style: italic;
}

.message-content p {
  margin: 0.4em 0;
}
</style>
