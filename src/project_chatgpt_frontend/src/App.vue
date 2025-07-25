<script setup>
import { ref, watch, onMounted, nextTick } from 'vue';
import { login, chatWithBackend, loginStatus } from './main.js';

const messages = ref([]);
const userInput = ref('');
const isLoggedIn = ref(false);
const showSidebar = ref(false);
const endOfMessages = ref(null);

async function sendMessage() {
  if (!userInput.value.trim()) return;

  messages.value.push({ role: 'user', content: userInput.value });

  try {
    const reply = await chatWithBackend(userInput.value);
    messages.value.push({ role: 'ai', content: reply });
  } catch (error) {
    messages.value.push({ role: 'ai', content: 'Błąd połączenia! ' + error.message });
  }

  userInput.value = '';
  await nextTick();
  scrollToBottom();
}

function scrollToBottom() {
  endOfMessages.value?.scrollIntoView({ behavior: 'smooth' });
}

onMounted(() => {
  const check = setInterval(() => {
    if (loginStatus.loggedIn) {
      isLoggedIn.value = true;
      clearInterval(check);
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
        <h3>Lista czatów</h3>
        <!-- Możesz tu dodać listę później -->
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
            <span class="message-content">{{ msg.content }}</span>
          </template>
          <template v-else>
            <span class="message-content">{{ msg.content }}</span>
            <span class="message-role">:USER</span>
          </template>
        </div>
        <div ref="endOfMessages" />
      </section>

      <!-- Input and buttons -->
      <section class="input-container">
        <input
          v-model="userInput"
          @keyup.enter="sendMessage"
          placeholder="Napisz wiadomość..."
          class="chat-input"
        />
        <button @click="sendMessage" class="btn-send">Wyślij</button>
        <button v-if="!isLoggedIn" @click="login" class="btn-login">Zaloguj się przez Internet Identity</button>
        <span v-if="isLoggedIn" class="logged-in-text">✅ Zalogowano</span>
      </section>
    </main>
  </div>
</template>

<style scoped>
.app-wrapper {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: 40px;
  background-color: #2c3e50;
  color: white;
  transition: width 0.3s;
  position: relative;
}

.sidebar.open {
  width: 200px;
}

.sidebar-toggle {
  position: absolute;
  right: -20px;
  top: 20px;
  width: 20px;
  height: 40px;
  background-color: #2c3e50;
  color: white;
  text-align: center;
  cursor: pointer;
  font-size: 20px;
  border-radius: 0 5px 5px 0;
}

.sidebar-content {
  padding: 10px;
}

.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: #f9f9f9;
  padding: 1rem;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  margin-bottom: 1rem;
}

.message {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.ai-message {
  justify-content: flex-start;
}

.user-message {
  justify-content: flex-end;
}

.message-role {
  font-weight: bold;
  margin: 0 5px;
}

.input-container {
  display: flex;
  gap: 0.5rem;
}

.chat-input {
  flex: 1;
  padding: 0.5rem;
}

.btn-send, .btn-login {
  padding: 0.5rem 1rem;
}

.logged-in-text {
  align-self: center;
  font-size: 0.9rem;
  color: green;
}
</style>